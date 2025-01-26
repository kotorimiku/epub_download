<template>
    <div class="config">
        <div class="form-item">
            <label>URL</label>
            <button @click="urlBaseChange" class="btn">{{ urlBase }}</button>
        </div>
        <div class="form-item">
            <label>下载间隔</label>
            <input type="number" v-model="sleepTime" placeholder="请输入下载间隔" class="input-box" />
        </div>
        <div class="form-item">
            <label>Cookie</label>
            <input type="text" v-model="cookie" placeholder="请输入cookie" class="input-box" />
        </div>
        <div class="form-item">
            <label>保存路径</label>
            <input type="text" v-model="outputPath" placeholder="请输入保存路径" class="input-box" />
        </div>
        <div class="form-item">
            <label>命名方式</label>
            <button class="btn" @click="changeLabel" title="影响命名方式，为true时，会添加[book_id]和[volume_no]到文件名中">{{
                addNumber ? '添加序号' : '取消序号' }}</button>
        </div>
        <div class="form-item">
            <label>是否添加目录页</label>
            <button class="btn" @click="changeCatalog">{{
                addCatalog ? '添加目录页' : '取消目录页' }}</button>
        </div>
        <!-- 新增保存配置按钮 -->
        <div class="form-item">
            <button @click="saveConfig" class="btn save-btn">保存配置</button>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useToast } from "vue-toastification";

const addNumber = ref(false)
const urlBase = ref<string>("")
const sleepTime = ref<number>(8)
const cookie = ref<string>("")
const outputPath = ref<string>("")
const addCatalog = ref(false)
const toast = useToast();

const changeLabel = () => {
    addNumber.value = !addNumber.value
}

const changeCatalog = () => {
    addCatalog.value = !addCatalog.value
}

const urlBaseChange = () => {
    if (urlBase.value === "https://www.bilinovel.com") {
        urlBase.value = "https://tw.linovelib.com"
    } else {
        urlBase.value = "https://www.bilinovel.com"
    }
}

const saveConfig = () => {
    invoke('save_config', {
        config: {
            output_path: outputPath.value,
            add_number: addNumber.value,
            cookie: cookie.value,
            sleep_time: sleepTime.value,
            url_base: urlBase.value,
        }
    }).then((res: any) => {
        console.log("配置已保存", res)
        toast.success('配置已保存', {
            timeout: 1000,
        });
    }).catch((err: any) => {
        console.error("保存配置失败", err)
        toast.error('保存配置失败', {
            timeout: 1000,
        });
    })
}

onMounted(() => {
    invoke('get_config_vue').then((res: any) => {
        if (res) {
            urlBase.value = res.url_base
            sleepTime.value = res.sleep_time
            cookie.value = res.cookie
            addNumber.value = res.add_number
            outputPath.value = res.output_path
        }
        if (cookie.value === "") {
            toast.error('您还没配置cookie，请先配置 Cookie', {
                timeout: 3000,
            });
        }
    }).catch(() => {
        console.log("获取配置失败")
        toast.error('获取配置失败', {
            timeout: 1000,
        });
    })
})
</script>

<style scoped>
/* 配置项容器 */
.config {
    width: 300px;
    padding: 20px;
    background-color: #f9f9f9;
    border-radius: 8px;
    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
    margin: 20px auto;
}

/* 表单项样式 */
.form-item {
    margin-bottom: 15px;
    display: flex;
    align-items: center;
    justify-content: space-between;
}

label {
    font-weight: bold;
    font-size: 14px;
    width: 80px;
    text-align: left;
}

/* 输入框样式 */
.input-box {
    width: 170px;
    padding: 8px;
    font-size: 14px;
    border-radius: 5px;
    border: 1px solid #ddd;
    margin-left: 10px;
}

/* 按钮样式 */
.btn {
    padding: 8px 16px;
    font-size: 14px;
    background-color: #ffffff;
    border: 1px solid #ddd;
    border-radius: 5px;
    cursor: pointer;
    transition: background-color 0.3s ease;
    margin-left: 10px;
}

.btn:hover {
    background-color: #f1f1f1;
}

.save-btn {
    width: 100%;
    background-color: #4CAF50;
    color: white;
    font-weight: bold;
}

.save-btn:hover {
    background-color: #45a049;
}

.disabled {
    background-color: #e0e0e0;
    cursor: not-allowed;
}
</style>
