use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimerState {
    Running,
    Paused,
    Stopped,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum SessionType {
    Work,
    ShortBreak,
    LongBreak,
}

#[derive(Debug, Clone)]
pub struct PomodoroState {
    pub state: TimerState,
    pub session_type: SessionType,
    pub remaining: u32,
    pub work_sessions_completed: u32,
    pub work_duration: u32,
    pub short_break_duration: u32,
    pub long_break_duration: u32,
}

#[derive(Serialize, Clone)]
pub struct TickPayload {
    pub remaining: u32,
    pub session_type: SessionType, // 假设你的 SessionType 也 derive(serde::Serialize, Clone, Copy)
}

impl PomodoroState {
    pub fn new() -> Self {
        PomodoroState {
            // 在这里设置一个番茄钟最开始的、最合理的默认状态
            state: TimerState::Stopped,      // 初始状态是停止的
            session_type: SessionType::Work, // 第一个会话总是“工作”
            remaining: 25 * 60,              // 默认25分钟，要换算成秒
            work_sessions_completed: 0,
            work_duration: 25 * 60,       // 默认25分钟
            short_break_duration: 5 * 60, // 默认5分钟
            long_break_duration: 15 * 60, // 默认15分钟
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct SettingsPayload {
    pub work_duration: u32,
    pub short_break_duration: u32,
    pub long_break_duration: u32,
}

pub type AppState<'a> = tauri::State<'a, Arc<Mutex<PomodoroState>>>;

#[tauri::command]
pub fn start_timer(state: AppState) {
    let mut timer = state.lock().unwrap();

    // 将计时器状态设置为运行中
    timer.state = TimerState::Running;

    // 在后台打印日志，方便调试
    println!("Command `start_timer` called: State changed to Running");
}

#[tauri::command]
pub fn stop_timer(state: AppState) {
    let mut timer = state.lock().unwrap();
    timer.state = TimerState::Stopped;
    println!("Command `stop_timer` called: State changed to Stopped");
}

#[tauri::command]
pub fn pause_timer(state: AppState) {
    let mut timer = state.lock().unwrap();
    timer.state = TimerState::Paused;
    println!("Command `pause_timer` called: State changed to Paused");
}

#[tauri::command]
pub fn reset_timer(state: AppState, app_handle: AppHandle) {
    let mut timer = state.lock().unwrap();
    timer.state = TimerState::Stopped;
    match timer.session_type {
        SessionType::Work => {
            timer.remaining = timer.work_duration;
            timer.session_type = SessionType::Work;
        }
        SessionType::ShortBreak => {
            timer.remaining = timer.short_break_duration;
            timer.session_type = SessionType::ShortBreak;
        }
        SessionType::LongBreak => {
            timer.remaining = timer.long_break_duration;
            timer.session_type = SessionType::LongBreak;
        }
    }

    println!("Command `reset_timer` called: State changed to Stopped, time has been reset.");

    let payload = TickPayload {
        remaining: timer.remaining,
        session_type: timer.session_type,
    };

    app_handle.emit("tick", payload).unwrap();
}

#[tauri::command]
pub fn get_setting(state: AppState) -> SettingsPayload {
    let timer = state.lock().unwrap();
    SettingsPayload {
        work_duration: timer.work_duration / 60,
        short_break_duration: timer.short_break_duration / 60,
        long_break_duration: timer.long_break_duration / 60,
    }
}

#[tauri::command]
pub fn update_setting(settings: SettingsPayload, state: AppState) {
    let mut timer = state.lock().unwrap();

    // 从前端接收分钟，转换为秒进行存储
    timer.work_duration = settings.work_duration * 60;
    timer.short_break_duration = settings.short_break_duration * 60;
    timer.long_break_duration = settings.long_break_duration * 60;

    println!("Settings updated!");
}
