use std::sync::Arc;

use vulkano::{app_info_from_cargo_toml, Version};
use vulkano::device::{Device, Queue};
use vulkano::device::DeviceExtensions;
use vulkano::device::Features;
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType, QueueFamily};
use vulkano::image::{AttachmentImage, ImageUsage, SwapchainImage};
use vulkano::image::view::ImageView;
use vulkano::instance::Instance;
use vulkano::swapchain::{ColorSpace, FullscreenExclusive, PresentMode, Surface, SurfaceTransform, Swapchain};
use vulkano_win::VkSurfaceBuild;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

/// Final render target (swap chain image)
pub type FinalImageView = Arc<ImageView<SwapchainImage<Window>>>;
/// Other intermediate render targets
pub type InterimImageView = Arc<ImageView<AttachmentImage>>;

pub(crate) struct Renderer {
    _instance: Arc<Instance>,
    device: Arc<Device>,
    surface: Arc<Surface<Window>>,
    queue: Arc<Queue>,
    swap_chain: Arc<Swapchain<Window>>,
    image_index: usize,
    final_views: Vec<FinalImageView>,
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

        Self {
            _instance: instance,
            device,
            surface,
            queue,
            swap_chain,
            image_index: 0,
            final_views,
        }
    }

    /// Creates vulkan device with required queue families and required extensions.
    fn create_device(physical: PhysicalDevice, surface: Arc<Surface<Window>>) -> (Arc<Device>, Arc<Queue>) {
        let queue_family = physical.queue_families()
            .find(|&q: &QueueFamily| q.supports_graphics() && surface.is_supported(q).unwrap_or(false))
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
        ).expect("failed to create device");

        let queue = queues.next().unwrap();

        (device, queue)
    }

    fn create_swap_chain(
        physical: PhysicalDevice,
        surface: Arc<Surface<Window>>,
        device: Arc<Device>,
        queue: Arc<Queue>,
    ) -> (Arc<Swapchain<Window>>, Vec<FinalImageView>) {
        let caps = surface.capabilities(physical)
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
            .present_mode(PresentMode::Fifo)
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
}
