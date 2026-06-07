import { watch, onMounted, onUnmounted } from "vue";

import { useSettingStore } from "../stores/settingStore";



const THEME_CLASS_MAP = {

  light: "theme-light",

  dark: "theme-dark",

  sepia: "theme-sepia",

  green: "theme-green",

  parchment: "theme-parchment",

  bluegray: "theme-bluegray",

  warm: "theme-warm",

  nightblue: "theme-nightblue",

};



/**

 * 全局主题 + 跟随系统（所有背景主题同步作用于 shell / 设置页 / 阅读页）

 */

export function useTheme() {

  const settingStore = useSettingStore();

  let mediaQuery = null;



  function syncSystemDark() {

    if (mediaQuery) {

      settingStore.systemDark = mediaQuery.matches;

    }

  }



  function applyBodyTheme() {

    document.body.classList.remove(

      "theme-light",

      "theme-dark",

      "theme-sepia",

      "theme-green",

      "theme-parchment",

      "theme-bluegray",

      "theme-warm",

      "theme-nightblue"

    );

    if (settingStore.followSystem) {

      if (settingStore.systemDark) document.body.classList.add("theme-dark");

      return;

    }

    const cls = THEME_CLASS_MAP[settingStore.theme];

    if (cls) {

      document.body.classList.add(cls);

    }

  }



  function onSystemChange() {

    syncSystemDark();

    applyBodyTheme();

  }



  onMounted(() => {

    mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");

    syncSystemDark();

    mediaQuery.addEventListener("change", onSystemChange);

    applyBodyTheme();

  });



  onUnmounted(() => {

    mediaQuery?.removeEventListener("change", onSystemChange);

  });



  watch(

    () => [settingStore.theme, settingStore.followSystem, settingStore.systemDark],

    () => applyBodyTheme()

  );

}

