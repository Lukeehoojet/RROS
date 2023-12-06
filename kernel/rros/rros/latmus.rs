#![no_std]
#![feature(allocator_api, global_asm)]
use crate::{
    thread::{self, rros_sleep, KthreadRunner}, timer::{self, rros_stop_timer}, clock,
};
use crate::bindings::ETIMEDOUT;
use kernel::prelude::*;
use kernel::{c_str, ktime, c_types};
// use rros::thread_test::KthreadRunner;

static mut kthread_runner_1: KthreadRunner = KthreadRunner::new_empty();

fn kthread_handler(ptr: *mut c_types::c_void) {
    // thread::rros_sleep(10000000000);
    let k_runner = unsafe{&mut *(ptr as *mut KthreadRunner)};
// void kthread_handler(void *arg)
// {
// 	struct kthread_runner *k_runner = arg;
// 	ktime_t now;
// 	int ret = 0;
    // TODO: add the ret value
    let kernel_lantency = [0; 1000];
    let mut ret = 0;

    loop {
        // TODO: add the should stop function
        // if thread::should_stop() {
        //     break;
        // }

        // TODO: add the wait flag function
        // ret = rros_wait_flag(&k_runner->barrier);
        // if (ret)
        //     break;

        // TODO: change the runner period flag when change
        thread::rros_set_period(unsafe{&mut clock::RROS_MONO_CLOCK},
                k_runner.1 as i64,
                k_runner.2.period as i64, 1);
        
        // TODO: error handle
        // if (ret)
        //     break;

        for i in 0..50 {
            pr_info!("I'm going to wait\n");
            thread::rros_wait_period();
            pr_info!("I'm in the loop\n");
            // if (ret && ret != -ETIMEDOUT) {
            //     // done_sampling(&k_runner.runner, ret);
            //     rros_stop_kthread(&k_runner.kthread);
            //     return;
            // }

            let now = unsafe{clock::rros_read_clock(&clock::RROS_MONO_CLOCK)};
            // if (k_runner.runner.add_sample(&k_runner.runner, now)) {
            if (add_measurement_sample(k_runner, now) == 1) {
                unsafe{thread::rros_set_period(&mut clock::RROS_MONO_CLOCK, 0, 0, 0);}
                break;
            }
        }

        break;
    }

    pr_info!("k_runner.2.state.min_latency: {}\n", k_runner.2.state.min_latency);
    pr_info!("k_runner.2.state.max_latency: {}\n", k_runner.2.state.max_latency);
    pr_info!("k_runner.2.state.avg_latency: {}\n", k_runner.2.state.avg_latency);
// 	for (;;) {
// 		if (rros_kthread_should_stop())
// 			break;

// 		ret = rros_wait_flag(&k_runner->barrier);
// 		if (ret)
// 			break;

// 		ret = rros_set_period(&rros_mono_clock,
// 				k_runner->start_time,
// 				k_runner->runner.period);
// 		if (ret)
// 			break;

// 		for (;;) {
// 			ret = rros_wait_period(NULL);
// 			if (ret && ret != -ETIMEDOUT)
// 				goto out;

// 			now = rros_read_clock(&rros_mono_clock);
// 			if (k_runner->runner.add_sample(&k_runner->runner, now)) {
// 				rros_set_period(NULL, 0, 0);
// 				break;
// 			}
// 		}
// 	}
// out:
// 	done_sampling(&k_runner->runner, ret);
// 	rros_stop_kthread(&k_runner->kthread);
// }
}

pub fn test_latmus() {
    unsafe{
        kthread_runner_1.init(Box::try_new(move || {
        let now = unsafe{clock::rros_read_clock(&clock::RROS_MONO_CLOCK)};
        unsafe{        
            kthread_runner_1.1 = now as u64;
            kthread_runner_1.2.period = 700000000;
            kthread_runner_1.2.state.ideal = now as u64;
            kthread_runner_1.2.state.offset = 0;
        }
        kthread_handler(&mut kthread_runner_1 as *mut KthreadRunner as *mut c_types::c_void);
    }).unwrap());
        kthread_runner_1.run(c_str!("latmus_thread"));
    }
}

fn add_measurement_sample(runner: &mut KthreadRunner, timestamp: ktime::KtimeT) -> i32{
    let mut period = runner.2.period as i64;
    let mut state = &mut runner.2.state;
    let mut delta = ktime::ktime_to_ns(ktime::ktime_sub(timestamp, state.ideal as i64)) as u64;
    pr_info!("the delta is {}\n", delta);
    pr_info!("the offset is {}\n", timestamp);
    pr_info!("the ideal is {}\n", state.ideal);
	let offset_delta = (delta - state.offset) as u64;

    if (offset_delta < state.min_latency) {
        state.min_latency = offset_delta;
    } else if (offset_delta > state.max_latency) {
        state.max_latency = offset_delta;
    } 

    pr_info!("the offset_delta is {}\n", offset_delta);
    pr_info!("the avg_latency is {}\n", state.avg_latency);
    state.avg_latency += offset_delta;
    state.ideal = ktime::ktime_add(state.ideal as i64, period) as u64;
    // else if offset_delta > state.allmax_lat {
        // state.allmax_lat = offset_delta;
        // trace_rros_latspot(offset_delta);
        // trace_rros_trigger("latmus");
    // }

    while (delta > 0 && delta > (ktime::ktime_to_ns(period) as u64)) { /* period > 0 */
        // let pexpect_ticks = unsafe{(*timer.locked_data().get()).get_pexpect_ticks() + 1};
        // unsafe{(*timer.locked_data().get()).set_pexpect_ticks(pexpect_ticks);}
		state.ideal = ktime::ktime_add(state.ideal as i64, period) as u64;
		delta -= ktime::ktime_to_ns(period) as u64;
	}

    0
} 

// static int add_measurement_sample(struct latmus_runner *runner,
//     ktime_t timestamp)
// {
// struct runner_state *state = &runner->state;
// ktime_t period = runner->period;
// int delta, cell, offset_delta;

// /* Skip samples in warmup time. */
// if (runner->warmup_samples < runner->warmup_limit) {
// runner->warmup_samples++;
// state->ideal = ktime_add(state->ideal, period);
// return 0;
// }

// delta = (int)ktime_to_ns(ktime_sub(timestamp, state->ideal));
// offset_delta = delta - state->offset;
// if (offset_delta < state->min_lat)
// state->min_lat = offset_delta;
// if (offset_delta > state->max_lat)
// state->max_lat = offset_delta;
// if (offset_delta > state->allmax_lat) {
// state->allmax_lat = offset_delta;
// trace_rros_latspot(offset_delta);
// trace_rros_trigger("latmus");
// }

// if (runner->histogram) {
// cell = (offset_delta < 0 ? -offset_delta : offset_delta) / 1000; /* us */
// if (cell >= runner->hcells)
// cell = runner->hcells - 1;
// runner->histogram[cell]++;
// }

// state->sum += offset_delta;
// state->ideal = ktime_add(state->ideal, period);

// while (delta > 0 &&
// (unsigned int)delta > ktime_to_ns(period)) { /* period > 0 */
// state->overruns++;
// state->ideal = ktime_add(state->ideal, period);
// delta -= ktime_to_ns(period);
// }

// if (++state->cur_samples >= state->max_samples)
// send_measurement(runner);

// return 0;	/* Always keep going. */
// }


// TODO: move this to a file
// struct Latmus;

// impl KernelModule for Latmus {
//     fn init() -> Result<Self> {
//         // unsafe{Arc::try_new(SpinLock::new(rros_thread::new().unwrap())).unwrap()},
//         unsafe{
//             kthread_runner_1.init(Box::try_new(move || {
//                 kthread_handler(&mut kthread_runner_1 as *mut KthreadRunner as *mut c_types::c_void);
//             }).unwrap());
//             kthread_runner_1.run(c_str!("latmus_thread"));
//         }

//         pr_info!("Hello world from latmus!\n");
//         Ok(Rros)
//     }
// }

// impl Drop for Rros {
//     fn drop(&mut self) {
//         pr_info!("Bye world from latmus!\n");
//     }
// }