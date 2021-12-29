use std::sync::Arc;

use vulkano::{app_info_from_cargo_toml, swapchain, sync, Version};
use vulkano::device::{Device, Queue};
use vulkano::device::DeviceExtensions;
use vulkano::device::Features;
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType, QueueFamily};
use vulkano::format::Format;
use vulkano::image::{
    AttachmentImage, ImageAccess, ImageUsage, ImageViewAbstract, SampleCount, SwapchainImage,
};
use vulkano::image::view::ImageView;
use vulkano::instance::Instance;
use vulkano::swapchain::{
    AcquireError, ColorSpace, FullscreenExclusive, PresentMode, Surface, SurfaceTransform,
    Swapchain, SwapchainCreationError,
};
use vulkano::sync::{FlushError, GpuFuture};
use vulkano_win::VkSurfaceBuild;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

use crate::RayMarchingComputePipeline;
use crate::renderer::render_pass_place_over_frame::RenderPassPlaceOverFrame;

mod pixels_draw_pipeline;
mod render_pass_place_over_frame;

/// Final render target (swap chain image)
pub(crate) type FinalImageView = Arc<ImageView<SwapchainImage<Window>>>;
/// Other intermediate render targets
pub(crate) type InterimImageView = Arc<ImageView<AttachmentImage>>;

pub(crate) struct Renderer {
    _instance: Arc<Instance>,
    device: Arc<Device>,
    surface: Arc<Surface<Window>>,
    queue: Arc<Queue>,
    swap_chain: Arc<Swapchain<Window>>,
    image_index: usize,
    final_views: Vec<FinalImageView>,
    interim_view: InterimImageView,
    recreate_swap_chain: bool,
    previous_frame_end: Option<Box<dyn GpuFuture>>,

    render_pass: RenderPassPlaceOverFrame,
    // TODO: This should probably not be in the Renderer
    compute_pipeline: RayMarchingComputePipeline,
}

impl Renderer {
    pub(crate) fn new(event_loop: &EventLoop<()>) -> Self {
        // Builds an `ApplicationInfo` by looking at the content of the `Cargo.toml` file at
        // compile-time.
        let app_infos = app_info_from_cargo_toml!();

        let extensions = vulkano_win::required_extensions();

        // Create Vulkano instance
        let instance = Instance::new(Some(&app_infos), Version::V1_1, &extensions, None)
            .expect("failed to create instance");

        // Get the best available physical device
        let physical: PhysicalDevice = PhysicalDevice::enumerate(&instance)
            .min_by_key(|p| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
            })
            .expect("no device supporting vulkan available");
        println!("Using device {}", physical.properties().device_name);

        // Create window + rendering surface
        let surface = WindowBuilder::new()
            .with_title("Ray Marching Demo")
            .build_vk_surface(&event_loop, instance.clone())
            .unwrap();

        // Create device
        let (device, queue) = Self::create_device(physical, surface.clone());

        // Create swap chain
        let (swap_chain, final_views) =
            Self::create_swap_chain(physical, surface.clone(), device.clone(), queue.clone());

        let previous_frame_end = Some(sync::now(device.clone()).boxed());

        // Create render pass
        let image_format = final_views.first().unwrap().format();
        let image_dimensions = final_views.first().unwrap().image().dimensions();
        let render_pass = RenderPassPlaceOverFrame::new(queue.clone(), image_format);

        let compute_pipeline = RayMarchingComputePipeline::new(queue.clone());
        let interim_view = ImageView::new(
            AttachmentImage::multisampled_with_usage(
                device.clone(),
                image_dimensions.width_height(),
                SampleCount::Sample1,
                Format::R8G8B8A8_UNORM,
                ImageUsage {
                    sampled: true,
                    input_attachment: true,
                    storage: true,
                    ..ImageUsage::none()
                },
            )
            .unwrap(),
        )
        .unwrap();

        Self {
            _instance: instance,
            device,
            surface,
            queue,
            swap_chain,
            image_index: 0,
            final_views,
            interim_view,
            recreate_swap_chain: false,
            previous_frame_end,

            render_pass,
            compute_pipeline,
        }
    }

    /// Creates vulkan device with required queue families and required extensions.
    fn create_device(
        physical: PhysicalDevice,
        surface: Arc<Surface<Window>>,
    ) -> (Arc<Device>, Arc<Queue>) {
        let queue_family = physical
            .queue_families()
            .find(|&q: &QueueFamily| {
                q.supports_graphics() && surface.is_supported(q).unwrap_or(false)
            })
            .expect("failed to find a graphical queue family");

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::none()
        };

        let features = Features::none();

        let (device, mut queues) = Device::new(
            physical,
            &features,
            &physical.required_extensions().union(&device_extensions),
            [(queue_family, 0.5)].iter().cloned(),
        )
        .expect("failed to create device");

        let queue = queues.next().unwrap();

        (device, queue)
    }

    /// Creates swap chain and swap chain images
    fn create_swap_chain(
        physical: PhysicalDevice,
        surface: Arc<Surface<Window>>,
        device: Arc<Device>,
        queue: Arc<Queue>,
    ) -> (Arc<Swapchain<Window>>, Vec<FinalImageView>) {
        let caps = surface
            .capabilities(physical)
            .expect("failed to get surface capabilities");
        let dimensions: [u32; 2] = surface.window().inner_size().into();
        let alpha = caps.supported_composite_alpha.iter().next().unwrap();
        let format = caps.supported_formats[0].0;

        let (swap_chain, images) = Swapchain::start(device, surface)
            .num_images(caps.min_image_count)
            .format(format)
            .dimensions(dimensions)
            .usage(ImageUsage::color_attachment())
            .sharing_mode(&queue)
            .composite_alpha(alpha)
            .transform(SurfaceTransform::Identity)
            .present_mode(PresentMode::Immediate)
            .fullscreen_exclusive(FullscreenExclusive::Default)
            .clipped(true)
            .color_space(ColorSpace::SrgbNonLinear)
            .layers(1)
            .build()
            .unwrap();

        let images = images
            .into_iter()
            .map(|image| ImageView::new(image).unwrap())
            .collect::<Vec<_>>();

        (swap_chain, images)
    }

    fn recreate_swap_chain(&mut self) {
        let dimensions: [u32; 2] = self.surface.window().inner_size().into();
        let (new_swap_chain, new_images) =
            match self.swap_chain.recreate().dimensions(dimensions).build() {
                Ok(result) => result,
                Err(SwapchainCreationError::UnsupportedDimensions) => {
                    println!(
                        "{}",
                        SwapchainCreationError::UnsupportedDimensions.to_string()
                    );
                    return;
                }
                Err(e) => panic!("Failed to recreate swap chain: {:?}", e),
            };

        self.swap_chain = new_swap_chain;
        self.final_views = new_images
            .into_iter()
            .map(|image| ImageView::new(image).unwrap())
            .collect();

        self.recreate_swap_chain = false;
    }

    fn start_frame(&mut self) -> Result<Box<dyn GpuFuture>, AcquireError> {
        // Recreate swap chain if needed (i.e. after window resized, or swap chain is outdated)
        if self.recreate_swap_chain {
            self.recreate_swap_chain();
        }

        // Acquire next image in the swap chain
        let (image_index, suboptimal, acquire_future) =
            match swapchain::acquire_next_image(self.swap_chain.clone(), None) {
                Ok(result) => result,
                Err(AcquireError::OutOfDate) => {
                    self.recreate_swap_chain = true;
                    return Err(AcquireError::OutOfDate);
                }
                Err(e) => panic!("Failed to acquire next image: {:?}", e),
            };
        if suboptimal {
            self.recreate_swap_chain = true;
        }

        self.image_index = image_index;

        let future = self.previous_frame_end.take().unwrap().join(acquire_future);
        Ok(future.boxed())
    }

    fn finish_frame(&mut self, after_future: Box<dyn GpuFuture>) {
        let future = after_future
            .then_swapchain_present(
                self.queue.clone(),
                self.swap_chain.clone(),
                self.image_index,
            )
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                // Prevent OutOfMemory error on Nvidia :(
                // https://github.com/vulkano-rs/vulkano/issues/627
                match future.wait(None) {
                    Ok(x) => x,
                    Err(err) => println!("{:?}", err),
                }
                self.previous_frame_end = Some(future.boxed());
            }
            Err(FlushError::OutOfDate) => {
                self.recreate_swap_chain = true;
                self.previous_frame_end = Some(sync::now(self.device.clone()).boxed());
            }
            Err(e) => {
                println!("Failed to flush future: {:?}", e);
                self.previous_frame_end = Some(sync::now(self.device.clone()).boxed());
            }
        }
    }

    // TODO: Should not live here?
    pub(crate) fn render(&mut self, t: f32) {
        // Clean-up, to avoid memory issues
        self.previous_frame_end.as_mut().unwrap().cleanup_finished();

        let acquire_future = self.start_frame().unwrap();

        let target_image = self.interim_view.clone();
        let compute_future = self
            .compute_pipeline
            .compute(target_image.clone(), t)
            .join(acquire_future);

        let render_pass_future = self
            .render_pass
            .render(
                compute_future,
                target_image,
                self.final_views[self.image_index].clone(),
            )
            .then_signal_fence_and_flush()
            .unwrap()
            .boxed();

        self.finish_frame(render_pass_future);
    }
}
