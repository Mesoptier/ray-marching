use vulkano::{app_info_from_cargo_toml, Version};
use vulkano::device::Device;
use vulkano::device::DeviceExtensions;
use vulkano::device::Features;
use vulkano::device::physical::{PhysicalDevice, QueueFamily};
use vulkano::instance::{Instance, InstanceExtensions};

fn main() {
    // Builds an `ApplicationInfo` by looking at the content of the `Cargo.toml` file at
    // compile-time.
    let app_infos = app_info_from_cargo_toml!();

    let extensions = InstanceExtensions::none();

    let instance = Instance::new(Some(&app_infos), Version::V1_1, &extensions, None)
        .expect("failed to create vulkan instance");

    let physical: PhysicalDevice = PhysicalDevice::enumerate(&instance).next()
        .expect("no device supporting vulkan available");

    let queue_family = physical.queue_families()
        .find(|q: &QueueFamily| q.supports_graphics())
        .expect("couldn't find a graphical queue family");

    let (device, mut queues) = Device::new(
        physical,
        &Features::none(),
        &DeviceExtensions::none(),
        [(queue_family, 0.5)].into_iter()
    ).expect("failed to create device");

    let queue = queues.next().unwrap();
}
