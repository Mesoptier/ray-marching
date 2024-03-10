use std::sync::Arc;

use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType, QueueFamily};
use vulkano::device::{Device, Queue};
use vulkano::device::{DeviceCreateInfo, DeviceExtensions, QueueCreateInfo};
use vulkano::format::Format;
use vulkano::image::view::{ImageView, ImageViewCreationError};
use vulkano::image::{
    AttachmentImage, ImageAccess, ImageUsage, ImageViewAbstract, SampleCount, SwapchainImage,
};
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::swapchain::{
    AcquireError, Surface, Swapchain, SwapchainCreateInfo, SwapchainCreationError,
};
use vulkano::sync::{FlushError, GpuFuture};
use vulkano::{swapchain, sync};
use vulkano_win::VkSurfaceBuild;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

use crate::renderer::render_pass_place_over_frame::RenderPassPlaceOverFrame;
use crate::RayMarchingComputePipeline;

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
    swapchain: Arc<Swapchain<Window>>,
    image_index: usize,
    final_views: Vec<FinalImageView>,
    interim_view: InterimImageView,
    recreate_swapchain: bool,
    previous_frame_end: Option<Box<dyn GpuFuture>>,

    render_pass: RenderPassPlaceOverFrame,
    // TODO: This should probably not be in the Renderer
    compute_pipeline: RayMarchingComputePipeline,
}

impl Renderer {
    pub(crate) fn new(event_loop: &EventLoop<()>) -> Self {
        let instance_create_info = InstanceCreateInfo {
            enabled_extensions: vulkano_win::required_extensions(),
            ..InstanceCreateInfo::application_from_cargo_toml()
        };

        // Create Vulkano instance
        let instance = Instance::new(instance_create_info).expect("failed to create instance");

        // Get the best available physical device
        let physical_device: PhysicalDevice = PhysicalDevice::enumerate(&instance)
            .min_by_key(|p| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
            })
            .expect("no device supporting vulkan available");
        println!("Using device {}", physical_device.properties().device_name);

        // Create window + rendering surface
        let surface = WindowBuilder::new()
            .with_title("Ray Marching Demo")
            .build_vk_surface(&event_loop, instance.clone())
            .unwrap();

        // Create device
        let (device, queue) = Self::create_device(physical_device, surface.clone());

        // Create swap chain
        let (swapchain, final_views) =
            Self::create_swapchain(physical_device, surface.clone(), device.clone());

        let previous_frame_end = Some(sync::now(device.clone()).boxed());

        // Create render pass
        let image_format = final_views.first().unwrap().format().unwrap();
        let image_dimensions = final_views.first().unwrap().image().dimensions();
        let render_pass = RenderPassPlaceOverFrame::new(queue.clone(), image_format);

        let compute_pipeline = RayMarchingComputePipeline::new(queue.clone());
        let interim_view =
            Self::create_interim_image_view(device.clone(), image_dimensions.width_height())
                .unwrap();

        Self {
            _instance: instance,
            device,
            surface,
            queue,
            swapchain,
            image_index: 0,
            final_views,
            interim_view,
            recreate_swapchain: false,
            previous_frame_end,

            render_pass,
            compute_pipeline,
        }
    }

    fn create_interim_image_view(
        device: Arc<Device>,
        dimensions: [u32; 2],
    ) -> Result<InterimImageView, ImageViewCreationError> {
        ImageView::new_default(
            AttachmentImage::multisampled_with_usage(
                device,
                dimensions,
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
    }

    /// Creates vulkan device with required queue families and required extensions.
    fn create_device(
        physical_device: PhysicalDevice,
        surface: Arc<Surface<Window>>,
    ) -> (Arc<Device>, Arc<Queue>) {
        let queue_family = physical_device
            .queue_families()
            .find(|&q: &QueueFamily| {
                q.supports_graphics() && q.supports_surface(&surface).unwrap_or(false)
            })
            .expect("failed to find a graphical queue family");

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::none()
        };

        let (device, mut queues) = Device::new(
            physical_device,
            DeviceCreateInfo {
                enabled_extensions: physical_device
                    .required_extensions()
                    .union(&device_extensions),
                queue_create_infos: vec![QueueCreateInfo::family(queue_family)],
                ..Default::default()
            },
        )
        .expect("failed to create device");

        let queue = queues.next().unwrap();

        (device, queue)
    }

    /// Creates swap chain and swap chain images
    fn create_swapchain(
        physical_device: PhysicalDevice,
        surface: Arc<Surface<Window>>,
        device: Arc<Device>,
    ) -> (Arc<Swapchain<Window>>, Vec<FinalImageView>) {
        let surface_capabilities = physical_device
            .surface_capabilities(&surface, Default::default())
            .expect("failed to get surface capabilities");

        let image_extent: [u32; 2] = surface.window().inner_size().into();
        let image_format = Some(
            physical_device
                .surface_formats(&surface, Default::default())
                .unwrap()[0]
                .0,
        );

        let (swapchain, images) = Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo {
                min_image_count: surface_capabilities.min_image_count,
                image_format,
                image_extent,
                image_usage: ImageUsage::color_attachment(),
                composite_alpha: surface_capabilities
                    .supported_composite_alpha
                    .iter()
                    .next()
                    .unwrap(),
                ..Default::default()
            },
        )
        .unwrap();

        let images = images
            .into_iter()
            .map(|image| ImageView::new_default(image).unwrap())
            .collect::<Vec<_>>();

        (swapchain, images)
    }

    fn recreate_swapchain(&mut self) {
        let image_extent: [u32; 2] = self.surface.window().inner_size().into();
        let (new_swapchain, new_images) = match self.swapchain.recreate(SwapchainCreateInfo {
            image_extent,
            ..self.swapchain.create_info()
        }) {
            Ok(result) => result,
            Err(err @ SwapchainCreationError::ImageExtentNotSupported { .. }) => {
                println!("{}", err);
                return;
            }
            Err(e) => panic!("Failed to recreate swap chain: {:?}", e),
        };

        self.swapchain = new_swapchain;
        self.final_views = new_images
            .into_iter()
            .map(|image| ImageView::new_default(image).unwrap())
            .collect();

        // Recreate image views
        self.interim_view =
            Self::create_interim_image_view(self.device.clone(), image_extent).unwrap();

        self.recreate_swapchain = false;
    }

    pub(crate) fn resize(&mut self) {
        self.recreate_swapchain = true;
    }

    fn start_frame(&mut self) -> Result<Box<dyn GpuFuture>, AcquireError> {
        // Recreate swap chain if needed (i.e. after window resized, or swap chain is outdated)
        if self.recreate_swapchain {
            self.recreate_swapchain();
        }

        // Acquire next image in the swap chain
        let (image_index, suboptimal, acquire_future) =
            match swapchain::acquire_next_image(self.swapchain.clone(), None) {
                Ok(result) => result,
                Err(AcquireError::OutOfDate) => {
                    self.recreate_swapchain = true;
                    return Err(AcquireError::OutOfDate);
                }
                Err(e) => panic!("Failed to acquire next image: {:?}", e),
            };
        if suboptimal {
            self.recreate_swapchain = true;
        }

        self.image_index = image_index;

        let future = self.previous_frame_end.take().unwrap().join(acquire_future);
        Ok(future.boxed())
    }

    fn finish_frame(&mut self, after_future: Box<dyn GpuFuture>) {
        let future = after_future
            .then_swapchain_present(self.queue.clone(), self.swapchain.clone(), self.image_index)
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
                self.recreate_swapchain = true;
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
