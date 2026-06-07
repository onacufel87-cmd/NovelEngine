<template>

  <div class="app-shell" :class="{ 'app-shell--reading': isReading }">

    <AppSidebar v-if="!isReading" />



    <div class="app-body">

      <main class="app-main" :class="{ 'app-main--reading': isReading }">

        <RouterView v-slot="{ Component }">

          <Transition name="page-fade" mode="out-in">

            <component :is="Component" class="route-page" />

          </Transition>

        </RouterView>

      </main>



      <AppMobileNav v-if="!isReading" />

    </div>

    <!-- 首次启动引导（设置加载完成后判断） -->
    <WelcomeWizard
      :visible="showOnboarding"
      @close="showOnboarding = false"
    />

  </div>

</template>



<script setup>

import { computed, onMounted, ref, watch } from "vue";

import { useRoute } from "vue-router";

import AppSidebar from "./components/Layout/AppSidebar.vue";

import AppMobileNav from "./components/Layout/AppMobileNav.vue";

import WelcomeWizard from "./components/Onboarding/WelcomeWizard.vue";

import { useTheme } from "./hooks/useTheme";

import { useSettingStore } from "./stores/settingStore";



const route = useRoute();

const settingStore = useSettingStore();



const isReading = computed(() => route.name === "read");

/** 首次引导弹窗：设置加载完成且未完成引导时显示 */
const showOnboarding = ref(false);

watch(
  () => settingStore.loaded,
  (loaded) => {
    if (loaded && !settingStore.onboardingCompleted) {
      showOnboarding.value = true;
    }
  },
  { immediate: true }
);



useTheme();



onMounted(() => {

  settingStore.loadFromDB();

});

</script>

