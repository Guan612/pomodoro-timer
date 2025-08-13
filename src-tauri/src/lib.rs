use std::sync::{Arc, Mutex};

use tauri::{App, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

use crate::timer::{PomodoroState, SessionType, TickPayload, TimerState};

// Learn more about Tauri commands at https://tauri.app/develop/calling-ru
mod timer;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state_timer = Arc::new(Mutex::new(timer::PomodoroState::new()));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .manage(state_timer.clone())
        .invoke_handler(tauri::generate_handler![
            timer::start_timer,
            timer::pause_timer,
            timer::stop_timer,
            timer::reset_timer,
            timer::get_setting,
            timer::update_setting,
        ])
        .setup(move |app| {
            setup_background_thread(app);
            // <-- 重点3：setup 闭包需要返回一个 Ok(())，表示“计划执行成功”
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup_background_thread(app: &mut App) {
    let app_handle = app.handle().clone();
    let state: tauri::State<Arc<Mutex<PomodoroState>>> = app.state();
    let state_clone = state.inner().clone();

    std::thread::spawn(move || {
        loop {
            let mut payload_to_emit: Option<TickPayload> = None;

            {
                // --- 上锁作用域开始 ---
                let mut timer = state_clone.lock().unwrap();

                // 只在计时器运行时，才执行核心逻辑
                if timer.state == TimerState::Running {
                    // 如果时间还没走完，就正常倒数
                    if timer.remaining > 0 {
                        timer.remaining -= 1;
                        //println!("remaining: {}", timer.remaining);

                        // 准备好这一秒的 tick 数据
                        payload_to_emit = Some(TickPayload {
                            remaining: timer.remaining,
                            session_type: timer.session_type,
                        });

                    // 如果时间正好走完，这是执行阶段切换的唯一时机！
                    } else {
                        // 1. 先停下计时器
                        timer.state = TimerState::Stopped;

                        // 2. 记录下刚刚结束的是哪个阶段，用于通知
                        let ended_session = timer.session_type;

                        // 3. 根据刚刚结束的阶段，决定下一个阶段是什么
                        match ended_session {
                            SessionType::Work => {
                                timer.work_sessions_completed += 1;
                                if timer.work_sessions_completed >= 4 {
                                    timer.session_type = SessionType::LongBreak;
                                    timer.remaining = timer.long_break_duration;
                                    timer.work_sessions_completed = 0;
                                } else {
                                    timer.session_type = SessionType::ShortBreak;
                                    timer.remaining = timer.short_break_duration;
                                }
                            }
                            SessionType::ShortBreak | SessionType::LongBreak => {
                                timer.session_type = SessionType::Work;
                                timer.remaining = timer.work_duration;
                            }
                        }

                        // 4. 发送桌面通知
                        let notification_title = match ended_session {
                            SessionType::Work => "工作结束！",
                            SessionType::ShortBreak => "短休息结束！",
                            SessionType::LongBreak => "长休息结束！",
                        };
                        app_handle
                            .notification()
                            .builder()
                            .title(notification_title)
                            .body("可以开始下一阶段了。")
                            .show()
                            .unwrap();

                        // 5. 准备好包含新阶段信息的 payload
                        payload_to_emit = Some(TickPayload {
                            remaining: timer.remaining,
                            session_type: timer.session_type,
                        });
                    }
                }
            } // --- 上锁作用域结束，锁在这里释放 ---

            // 在锁已释放的安全区域，处理事件发送
            if let Some(payload) = payload_to_emit {
                app_handle.emit("tick", payload).unwrap();
            }

            // 在完全无锁的状态下，让线程休眠
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }); // 线程在这里被创建并“发射”出去，它会自己在后台运行
}
