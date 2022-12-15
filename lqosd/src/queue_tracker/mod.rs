use std::time::{Duration, Instant};
use lqos_config::LibreQoSConfig;
use tokio::{task, time};
mod queue_reader;

fn track_queues() {
    let config = LibreQoSConfig::load().unwrap();
    let (download_queues, upload_queues) = if config.on_a_stick_mode {
        let num_cpus = unsafe {
            lqos_sys::libbpf_num_possible_cpus()
        } as u32;
        let half_cpus = num_cpus / 2;
        let upload_offset = half_cpus;
        let queues = queue_reader::read_tc_queues(&config.internet_interface).unwrap();
        // TODO: Divide into upload and download
        (queues.clone(), queues.clone())
    } else {
        (
            queue_reader::read_tc_queues(&config.isp_interface).unwrap(),
            queue_reader::read_tc_queues(&config.internet_interface).unwrap(),
        )
    };

    //println!("{:#?}", download_queues);
}

pub async fn spawn_queue_monitor() {
    let _ = task::spawn(async {
        let mut interval = time::interval(Duration::from_secs(10));

        loop {
            let now = Instant::now();
            let _ = task::spawn_blocking(move || {
                track_queues()
            })
            .await;
            let elapsed = now.elapsed();
            //println!("TC Reader tick consumed {:.4} seconds.", elapsed.as_secs_f32());
            if elapsed.as_secs_f32() < 10.0 {
                let duration = Duration::from_secs(10) - elapsed;
                //println!("Sleeping for {:.2} seconds", duration.as_secs_f32());
                tokio::time::sleep(duration).await;
            } else {
                interval.tick().await;
            }
        }
    });
}