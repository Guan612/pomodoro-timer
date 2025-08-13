<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from '@tauri-apps/api/event'

// 3. 控制计时器的命令
const startTimer = () => invoke('start_timer');
const pauseTimer = () => invoke('pause_timer');
const resetTimer = () => invoke('reset_timer');
const getSetting = () => invoke('get_setting');

// --- 计时器核心状态 ---
const remainingSeconds = ref(25 * 60);
const sessionType = ref('Work');
const totalSeconds = ref(25 * 60);

// --- 设置模态框状态 ---
const isSettingsOpen = ref(false); // 控制模态框是否显示
const workMinutes = ref(25);      // 设置中的“工作分钟”
const shortBreakMinutes = ref(5); // 设置中的“短休息分钟”
const longBreakMinutes = ref(15); // 设置中的“长休息分钟”


// --- 环形进度条计算属性 ---
const strokeWidth = 10;
const radius = 80;
const center = radius + strokeWidth;
const circumference = 2 * Math.PI * radius;
const progressPercentage = computed(() => {
    if (totalSeconds.value === 0) return 0;
    // 确保进度不会超过100%
    return Math.min((remainingSeconds.value / totalSeconds.value) * 100, 100);
});
const dashoffset = computed(() => {
    return circumference - (progressPercentage.value / 100) * circumference;
});

// --- UI 显示计算属性 ---
const formattedTime = computed(() => {
    const minutes = Math.floor(remainingSeconds.value / 60);
    const seconds = remainingSeconds.value % 60;
    return `${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`;
});
const progressColorClass = computed(() => {
    switch (sessionType.value) {
        case 'Work': return 'text-green-500';
        case 'ShortBreak': return 'text-blue-500';
        case 'LongBreak': return 'text-indigo-500';
        default: return 'text-gray-500';
    }
});

const gettimerSetting = async () => {
    const res = await getSetting();
    workMinutes.value = res.work_minutes;
    shortBreakMinutes.value = res.short_break_minutes;
    longBreakMinutes.value = res.long_break_minutes;
}

// --- 与后端交互 ---

// 1. 监听来自 Rust 的 tick 事件
onMounted(() => {
    listen('tick', (event) => {
        remainingSeconds.value = event.payload.remaining;
        sessionType.value = event.payload.session_type;
    });

    gettimerSetting()
});

// 2. 监视 sessionType 变化，更新总时长
watch(sessionType, (newType) => {
    switch (newType) {
        case 'Work': totalSeconds.value = workMinutes.value * 60; break;
        case 'ShortBreak': totalSeconds.value = shortBreakMinutes.value * 60; break;
        case 'LongBreak': totalSeconds.value = longBreakMinutes.value * 60; break;
    }
    // 如果会话改变时时间是0，手动将剩余时间也更新
    if (remainingSeconds.value === 0) {
        remainingSeconds.value = totalSeconds.value;
    }
});

// 4. 控制设置的命令
async function openSettings() {
    // 从后端获取当前设置并填充输入框
    const settings = await invoke('get_setting');
    workMinutes.value = settings.work_duration;
    shortBreakMinutes.value = settings.short_break_duration;
    longBreakMinutes.value = settings.long_break_duration;
    isSettingsOpen.value = true;
}

async function saveSettings() {
    // 将新设置发送给后端
    await invoke('update_setting', {
        settings: {
            work_duration: workMinutes.value,
            short_break_duration: shortBreakMinutes.value,
            long_break_duration: longBreakMinutes.value,
        }
    });
    // 保存后关闭模态框
    isSettingsOpen.value = false;
    // （推荐）保存设置后调用一次重置，让计时器立即采用新时长
    await resetTimer();
}

</script>

<template>
    <div class="relative p-4">
        <div class="flex flex-col items-center gap-8">
            <div class="relative flex items-center justify-center">
                <svg class="w-48 h-48 transform -rotate-90">
                    <circle class="text-gray-300 dark:text-gray-700" stroke="currentColor" :stroke-width="strokeWidth"
                        fill="transparent" :r="radius" :cx="center" :cy="center" />
                    <circle :class="progressColorClass" class="transition-all duration-300" stroke="currentColor"
                        :stroke-width="strokeWidth" stroke-linecap="round" fill="transparent" :r="radius" :cx="center"
                        :cy="center" :style="{ strokeDasharray: circumference, strokeDashoffset: dashoffset }" />
                </svg>
                <div class="absolute flex flex-col items-center">
                    <span class="text-4xl font-bold text-gray-700 dark:text-gray-200">{{ formattedTime }}</span>
                    <span class="text-sm text-gray-500">{{ sessionType }}</span>
                </div>
            </div>

            <div class="flex gap-4">
                <button @click="startTimer"
                    class="px-6 py-2 font-semibold text-white bg-green-500 rounded-md hover:bg-green-600">开始</button>
                <button @click="pauseTimer"
                    class="px-6 py-2 font-semibold text-white bg-yellow-500 rounded-md hover:bg-yellow-600">暂停</button>
                <button @click="resetTimer"
                    class="px-6 py-2 font-semibold text-white bg-red-500 rounded-md hover:bg-red-600">重置</button>
            </div>
        </div>

        <button @click="openSettings" class="absolute top-4 right-4 text-gray-400 hover:text-gray-600">
            <svg xmlns="http://www.w3.org/2000/svg" class="w-6 h-6" fill="none" viewBox="0 0 24 24"
                stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
        </button>

        <div v-if="isSettingsOpen"
            class="absolute inset-0 z-10 flex items-center justify-center bg-opacity-50">
            <div class="p-8 bg-white rounded-lg shadow-xl dark:bg-gray-800">
                <h2 class="mb-6 text-2xl font-bold text-gray-800 dark:text-gray-100">设置</h2>
                <div class="space-y-4">
                    <div class="flex items-center justify-between">
                        <label for="work" class="text-gray-600 dark:text-gray-300">工作时长 (分钟):</label>
                        <input id="work" type="number" v-model="workMinutes"
                            class="w-20 p-2 text-right border rounded-md dark:bg-gray-700 dark:border-gray-600 dark:text-white">
                    </div>
                    <div class="flex items-center justify-between">
                        <label for="short-break" class="text-gray-600 dark:text-gray-300">短休息 (分钟):</label>
                        <input id="short-break" type="number" v-model="shortBreakMinutes"
                            class="w-20 p-2 text-right border rounded-md dark:bg-gray-700 dark:border-gray-600 dark:text-white">
                    </div>
                    <div class="flex items-center justify-between">
                        <label for="long-break" class="text-gray-600 dark:text-gray-300">长休息 (分钟):</label>
                        <input id="long-break" type="number" v-model="longBreakMinutes"
                            class="w-20 p-2 text-right border rounded-md dark:bg-gray-700 dark:border-gray-600 dark:text-white">
                    </div>
                </div>
                <div class="flex justify-end mt-8 space-x-4">
                    <button @click="isSettingsOpen = false"
                        class="px-4 py-2 font-semibold text-gray-700 bg-gray-200 rounded-md hover:bg-gray-300 dark:bg-gray-600 dark:text-gray-200 dark:hover:bg-gray-500">取消</button>
                    <button @click="saveSettings"
                        class="px-4 py-2 font-semibold text-white bg-blue-500 rounded-md hover:bg-blue-600">保存</button>
                </div>
            </div>
        </div>

    </div>
</template>