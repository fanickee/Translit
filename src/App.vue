<script setup lang="ts">
import { onMounted, ref } from "vue";
import Button from "./components/ui/button/Button.vue";
// import Label from "@/components/ui/label/Label.vue";
// import Switch from '@/components/ui/switch/Switch.vue';
import { Toaster } from '@/components/ui/sonner'
import { toast } from 'vue-sonner'
import 'vue-sonner/style.css';
import Toggle from '@/components/ui/toggle/Toggle.vue';
import { useColorMode } from '@vueuse/core'
import { Sun, Moon, X, Minus, Pin, PinOff } from 'lucide-vue-next';
import {
  Menubar,
  MenubarContent,
  MenubarItem,
  MenubarCheckboxItem,
  MenubarSub,
  MenubarSubContent,
  MenubarSubTrigger,
  MenubarMenu,
  MenubarSeparator,
  MenubarShortcut,
  MenubarRadioGroup,
  MenubarRadioItem,
  MenubarTrigger,
} from '@/components/ui/menubar'
import { Window } from '@tauri-apps/api/window';
import { useSettingsStore } from "./stores/settings";
// import { saveWindowState, StateFlags, restoreStateCurrent } from '@tauri-apps/plugin-window-state';
/* global */
const settingsStore = useSettingsStore();
const appWindow = new Window('main');

/* dark mode */
const mode = useColorMode()
const _isDark = ref(mode.state.value == 'dark');
function toggleTheme() {
  console.log(_isDark.value);
  _isDark.value = !_isDark.value;
  if (_isDark.value) {
    mode.store.value = 'dark';
  } else {
    mode.store.value = 'light';
  }
}
import { TrayIcon, TrayIconOptions } from '@tauri-apps/api/tray';
import { Menu, MenuItem, PredefinedMenuItem, CheckMenuItem } from '@tauri-apps/api/menu';
import { defaultWindowIcon } from '@tauri-apps/api/app';
import { getVersion } from '@tauri-apps/api/app';

async function createTray() {
  const hotkeyItem = []
  for (const hotkey of settingsStore.settings.hotKeyList) {
    hotkeyItem.push(CheckMenuItem.new({
      text: hotkey.desc,
      checked: hotkey.isOpen,
      action() {
        settingsStore.enableShortcut(hotkey.use, !hotkey.isOpen, null)
      },
    }))
  }
  const version = await getVersion();
  const items = await Promise.all([
    ...hotkeyItem,
    PredefinedMenuItem.new({ item: 'Separator' }),
    CheckMenuItem.new({
      text: '获取后自动翻译',
      checked: settingsStore.settings.autoTrans,
      action: () => {
        settingsStore.settings.autoTrans = !settingsStore.settings.autoTrans
      },
    }),
    PredefinedMenuItem.new({ item: 'Separator' }),
    MenuItem.new({
      text: `版本 ${version}`,
      enabled: false,
    }),
    MenuItem.new({
      text: '退出',
      action: () => appWindow.close(),
    }),
    MenuItem.new({
      text: '打开',
      action: async () => {
        await appWindow.show()
        await settingsStore.winToTop()
      },
    }),
  ])

  const menu = await Menu.new({ items });
  const icon = await defaultWindowIcon()
  const options: TrayIconOptions = {
    menu,
    menuOnLeftClick: true,
    icon: icon ? icon : undefined,
    action: async (event) => {
      switch (event.type) {
        case 'DoubleClick':
          await appWindow.show()
          await settingsStore.winToTop()
          break;
      }
    },
  }
  await TrayIcon.new(options);
}

async function closeWindow() {
  // await saveWindowState(StateFlags.ALL);
  appWindow.hide()
}

import { check, Update } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { Progress } from '@/components/ui/progress'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  // AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'

const openUpdate = ref(false);
let update: Update | null = null;
const showProgress = ref(false);
const progress = ref(0);
async function checkUpdate() {
  if (showProgress.value) {
    return;
  }
  try {
    update = await check();
    if (!update) {
      toast.warning("无更新信息");
      return;
    }
  } catch (e) {
    toast.warning("无法获取更新信息");
    return;
  }

  if (update.currentVersion == update.version) {
    toast.warning("无需更新");
    return;
  }
  console.log(
    `found update ${update.version} from ${update.date} with notes ${update.body}`
  );
  let downloaded = 0;
  let contentLength = 0;
  showProgress.value = true;
  try {
    // alternatively we could also call update.download() and update.install() separately
    await update.downloadAndInstall((event) => {
      if (update == null)
        return
      switch (event.event) {
        case 'Started':
          toast.info(`开始下载: ${update.version}`);
          progress.value = 0;
          contentLength = event.data.contentLength ?? 0;
          console.log(`started downloading ${event.data.contentLength} bytes`);
          break;
        case 'Progress':
          downloaded += event.data.chunkLength;
          progress.value = downloaded * 100 / contentLength;
          // console.log(`downloaded ${downloaded} from ${contentLength}`);
          break;
        case 'Finished':
          progress.value = 100;
          console.log('download finished');
          break;
      }
    });
    console.log('update installed');
    openUpdate.value = true;
  } catch (e) {
    toast.error(`下载失败: ${e}`)
    console.log(e)
  }
  showProgress.value = false;
}

onMounted(async () => {
  console.log("mounted App")
  settingsStore.allBackendInit().finally(() => {
    createTray()
  })
})

</script>

<template>
  <div class="relative flex flex-col flex-nowrap justify-start h-full w-full bg-background">
    <div class="relative flex flex-row w-full justify-start align-middle p-2 border-b-2" data-tauri-drag-region>
      <Menubar>
        <MenubarMenu>
          <MenubarTrigger>设置</MenubarTrigger>
          <MenubarContent>
            <MenubarSub>
              <MenubarSubTrigger>Api</MenubarSubTrigger>
              <MenubarSubContent>
                <MenubarRadioGroup v-on:update:model-value="settingsStore.chooseApi"
                  :model-value="settingsStore.settings.defaultApi">
                  <MenubarRadioItem v-for="(api, _) in settingsStore.apiList" :value="api">
                    {{ api }}
                  </MenubarRadioItem>
                </MenubarRadioGroup>
              </MenubarSubContent>
            </MenubarSub>
            <MenubarSeparator />
            <MenubarSub>
              <MenubarSubTrigger>热键</MenubarSubTrigger>
              <MenubarSubContent>
                <MenubarCheckboxItem v-for="v in settingsStore.settings.hotKeyList" :model-value="v.isOpen"
                  @update:model-value="b => settingsStore.enableShortcut(v.use, b, null)">
                  {{ v.desc }}<MenubarShortcut>{{ v.keys }}</MenubarShortcut>
                </MenubarCheckboxItem>
                <MenubarSeparator />
                <MenubarCheckboxItem v-model:model-value="settingsStore.settings.autoTrans">获取后自动翻译
                </MenubarCheckboxItem>
              </MenubarSubContent>
            </MenubarSub>
            <MenubarSeparator />
            <MenubarItem @click="checkUpdate">
              检测更新
            </MenubarItem>
            <MenubarItem>
              <RouterLink to="/settings">更多设置</RouterLink>
            </MenubarItem>
          </MenubarContent>
        </MenubarMenu>
        <MenubarMenu>
          <MenubarTrigger>关于</MenubarTrigger>
          <MenubarContent>
            <MenubarItem>
              <RouterLink to="/about">版本信息</RouterLink>
            </MenubarItem>
          </MenubarContent>
        </MenubarMenu>
      </Menubar>
      <div class="absolute left-1/2 top-1/2 -translate-1/2 flex m-auto items-center justify-center">
        <p class="text-foreground text-lg text-center font-bold select-none" data-tauri-drag-region>Translit</p>
      </div>
      <div class="flex gap-0.5 ml-auto">
        <Button variant="ghost" @click="toggleTheme">
          <Sun v-if="_isDark" />
          <Moon v-else />
        </Button>
        <Toggle :model-value="settingsStore.pinState" v-on:update:model-value="v => settingsStore.setAlwaysTop(v)">
          <Pin v-if="settingsStore.pinState == false"></Pin>
          <PinOff v-else></PinOff>
        </Toggle>
        <Button variant="outline" size="icon" @click="appWindow.minimize()">
          <Minus />
        </Button>
        <Button variant="outline" size="icon" @click="closeWindow">
          <X />
        </Button>
      </div>
    </div>
    <router-view v-slot="{ Component }">
      <transition mode="out-in" name="slide-fade">
        <KeepAlive include="IndexView">
          <component :is="Component" />
        </KeepAlive>
      </transition>
    </router-view>
    <Progress :model-value="progress" v-if="showProgress"></Progress>
    <Toaster :close-button="true" position="bottom-left" />
    <AlertDialog :open="openUpdate">
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>更新完成是否重启?</AlertDialogTitle>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel @click="openUpdate = false">否</AlertDialogCancel>
          <AlertDialogAction @click="relaunch();openUpdate = false">是</AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  </div>
</template>

<style scoped></style>
<style>
html,
body {
  height: 100vh;
  width: 100vw;
  margin: 0;
  overflow: hidden;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.5s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.slide-fade-enter-active {
  transition: all 0.3s ease-out;
}

.slide-fade-leave-active {
  transition: all 0.5s cubic-bezier(1, 0.5, 0.8, 1);
}

.slide-fade-enter-from,
.slide-fade-leave-to {
  transform: translateY(20px);
  opacity: 0;
}
</style>