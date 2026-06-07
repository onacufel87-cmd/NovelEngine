<template>
  <Teleport to="body">
    <div v-if="visible" class="onboarding-overlay" @click.self="skip">
      <div class="onboarding-card" role="dialog" aria-modal="true" aria-labelledby="onboarding-title">
        <!-- 步骤指示 -->
        <div class="step-dots">
          <span
            v-for="(_, i) in steps"
            :key="i"
            class="dot"
            :class="{ active: i === stepIndex }"
          />
        </div>

        <div class="step-body">
          <div class="step-icon">{{ currentStep.icon }}</div>
          <h2 id="onboarding-title" class="step-title">{{ currentStep.title }}</h2>
          <p class="step-desc">{{ currentStep.desc }}</p>

          <ul v-if="currentStep.bullets?.length" class="step-bullets">
            <li v-for="(item, i) in currentStep.bullets" :key="i">{{ item }}</li>
          </ul>
        </div>

        <footer class="step-footer">
          <button type="button" class="btn-skip" @click="skip">跳过</button>
          <div class="footer-actions">
            <button
              v-if="stepIndex > 0"
              type="button"
              class="btn-secondary"
              @click="prev"
            >
              上一步
            </button>
            <button type="button" class="btn-primary" @click="next">
              {{ isLast ? "开始使用" : "下一步" }}
            </button>
          </div>
        </footer>
      </div>
    </div>
  </Teleport>
</template>

<script setup>
import { ref, computed } from "vue";
import { useRouter } from "vue-router";
import { useSettingStore } from "../../stores/settingStore";

const props = defineProps({
  visible: { type: Boolean, default: false },
});

const emit = defineEmits(["close"]);

const router = useRouter();
const settingStore = useSettingStore();
const stepIndex = ref(0);

const steps = [
  {
    icon: "📚",
    title: "欢迎使用小说引擎",
    desc: "这是一款本地优先的桌面阅读器。书架、书源与阅读进度都保存在你的电脑上，无需联网账号。",
    bullets: [
      "数据仅存本地，隐私可控",
      "支持 EPUB / TXT 离线导入",
      "内置 Gutenberg 等公版书源",
    ],
  },
  {
    icon: "🔍",
    title: "发现公版好书",
    desc: "我们已为你预置合法公版书源。在「发现」页搜索书名或浏览榜单，一键加入书架即可阅读。",
    bullets: [
      "发现页 → 全网搜索 / 榜单",
      "书源页可管理、订阅自定义规则",
      "高级用户可手动粘贴 JSON 规则",
    ],
  },
  {
    icon: "✨",
    title: "准备就绪",
    desc: "你可以从书架导入本地文件，或直接去发现页找第一本书。阅读时点击「目录」可随时弹出章节目录。",
    bullets: ["书架支持网格 / 列表切换", "阅读设置里可调字体、主题与简繁"],
  },
];

const currentStep = computed(() => steps[stepIndex.value]);
const isLast = computed(() => stepIndex.value >= steps.length - 1);

function prev() {
  if (stepIndex.value > 0) stepIndex.value -= 1;
}

function next() {
  if (isLast.value) {
    finish(true);
    return;
  }
  stepIndex.value += 1;
}

function skip() {
  finish(false);
}

/** 完成引导并持久化标记 */
function finish(goDiscover) {
  settingStore.updateSetting("onboardingCompleted", true);
  emit("close");
  if (goDiscover && stepIndex.value === steps.length - 1) {
    router.push("/discover");
  }
}
</script>

<style scoped>
.onboarding-overlay {
  position: fixed;
  inset: 0;
  z-index: 1000;
  background: rgba(20, 24, 20, 0.55);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
  animation: fade-in 0.25s ease;
}

@keyframes fade-in {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

.onboarding-card {
  width: 100%;
  max-width: 440px;
  background: var(--color-surface);
  border-radius: var(--radius-lg, 16px);
  box-shadow: var(--shadow-lg, 0 16px 48px rgba(0, 0, 0, 0.18));
  padding: 28px 28px 22px;
  animation: slide-in 0.3s ease;
}

@keyframes slide-in {
  from {
    transform: translateY(12px);
    opacity: 0;
  }
  to {
    transform: translateY(0);
    opacity: 1;
  }
}

.step-dots {
  display: flex;
  justify-content: center;
  gap: 8px;
  margin-bottom: 20px;
}

.dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--color-border);
  transition: background 0.2s, transform 0.2s;
}

.dot.active {
  background: var(--color-primary);
  transform: scale(1.15);
}

.step-body {
  text-align: center;
}

.step-icon {
  font-size: 2.4rem;
  margin-bottom: 12px;
}

.step-title {
  font-size: 1.25rem;
  font-weight: 600;
  margin-bottom: 10px;
  color: var(--color-text);
}

.step-desc {
  font-size: 0.92rem;
  line-height: 1.65;
  color: var(--color-muted);
  margin-bottom: 16px;
}

.step-bullets {
  text-align: left;
  list-style: none;
  padding: 14px 16px;
  margin: 0;
  background: var(--color-bg, #f9f7f4);
  border-radius: var(--radius-md);
  border: 1px solid var(--color-border-light);
}

.step-bullets li {
  font-size: 0.85rem;
  color: var(--color-text);
  padding: 5px 0;
  padding-left: 18px;
  position: relative;
}

.step-bullets li::before {
  content: "·";
  position: absolute;
  left: 4px;
  color: var(--color-primary);
  font-weight: bold;
}

.step-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 24px;
  gap: 12px;
}

.btn-skip {
  border: none;
  background: none;
  color: var(--color-muted);
  font-size: 0.85rem;
  cursor: pointer;
  padding: 8px 4px;
}

.btn-skip:hover {
  color: var(--color-text);
}

.footer-actions {
  display: flex;
  gap: 10px;
}

.btn-secondary,
.btn-primary {
  border: none;
  border-radius: var(--radius-pill, 999px);
  padding: 10px 20px;
  font-size: 0.88rem;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.15s;
}

.btn-secondary {
  background: var(--color-bg);
  color: var(--color-text);
  border: 1px solid var(--color-border);
}

.btn-primary {
  background: var(--color-primary);
  color: #fff;
}

.btn-primary:hover {
  filter: brightness(1.05);
}
</style>
