import { defineStore } from 'pinia'
import { reactive, ref, watch } from 'vue'
import { load, Store } from '@tauri-apps/plugin-store';
import { isRegistered, register, ShortcutEvent, ShortcutHandler, unregister } from '@tauri-apps/plugin-global-shortcut';

import { toast } from 'vue-sonner';
import { invoke } from '@tauri-apps/api/core';
import { Window } from '@tauri-apps/api/window';

export type ModifierKey = "CTRL" | "ALT" | "CMD" | "SHIFT"
const keySymbols: Record<string, string> = {
  SHIFT: '⇧',
  CTRL: '⌃',
  ALT: '⌥',
  CMD: '⌘',
  UP: '↑',
  DOWN: '↓',
  LEFT: '←',
  RIGHT: '→'
}
type StringPair = [string, string];
type Uses = "Select"|"Clip"
interface HotKey {
  isOpen: boolean,
  use: string,
  desc: string,
  keys: string
}

interface SettingStore {
  hotKeyList: HotKey[],
  defaultApi: string,
  autoTrans: boolean
}

export const useSettingsStore = defineStore('settings', ()=>{
  const settings = reactive<SettingStore>({
    hotKeyList: [],
    defaultApi: "",
    autoTrans: false
  })

  const apiList = ref<string[]>([])
  const apiLangs = ref<StringPair[]>([])
  let store: null | Store = null;

  function mapKey2Str(key: string) {
    if (key in keySymbols) {
      return keySymbols[key]
    } else {
      return key
    }
  }

  async function getSupportApi() {
    await invoke<string[]>('support_apis').then((apis) => {
      console.log(apis)
      apiList.value = apis;
    })
  }

  async function chooseApi(v:string) {
    console.log(v);
    try {
      await invoke("choose_api", {
        name: v,
        args: null
      })
      await invoke<StringPair[]>("support_lang").then((l)=>{
        apiLangs.value = l.reverse()
      })
    } catch(e) {
      toast.error(`failed to choose api ${v}`)
    }
  }

  async function allBackendInit() {
    console.log("call init")
    try {
      await getStoreSetting()
      await getSupportApi()
      if (!settings.defaultApi) {
        settings.defaultApi = apiList.value[0]
      }
      await chooseApi(settings.defaultApi)
      for (const v of settings.hotKeyList) {
        if (v.isOpen) {
          await enableShortcut(v.use as Uses, true, v.keys)
        }
      }
    } catch(e) {
      toast.error(`"init error:${e}"`)
    }
  }

  async function getStoreSetting() {
    try {
      store = await load('settings.json', { autoSave: true });
      if (!store) {
        throw Error("can not load store")
      }
      // await store.set("settings", {
      //   defaultApi: "",
      //   hotKeyList: [
      //     {
      //       use: "Select",
      //       isOpen: false,
      //       desc: "获取选中文本",
      //       keys: "ALT+X"
      //     },
      //     {
      //       use: "Clip",
      //       isOpen: false,
      //       desc: "获取粘贴文本",
      //       keys: "ALT+C"
      //     },
      //   ]
      // })
      let tmp = await store.get<SettingStore>("settings") ?? {
        defaultApi: "",
        hotKeyList: [
          {
            use: "Select",
            isOpen: false,
            desc: "获取选中文本",
            keys: "ALT+X"
          },
          {
            use: "Clip",
            isOpen: false,
            desc: "获取粘贴文本",
            keys: "ALT+C"
          },
        ],
        autoTrans: false
      }

      Object.assign(settings, tmp)
      watch(settings, async (n, _)=>{
        if (!store) {
          console.error("ignore store")
          return
        }
        console.log(settings)
        store.set("settings", n)
      })
    } catch (err: any) {
      console.error(err)
      toast.error(`fail to load settings: ${err}`)
    }
  }

  async function registerHotKey(keys:string, callback:ShortcutHandler) {
    console.log(`register ${keys} ing`)
    try {
      if (await isRegistered(keys)) {
        await unregister(keys)
      }
      console.log(await register(keys, callback))
    } catch(e) {
      toast.error(`set key err: ${e}`)
      return false
    }
    return true
  }

  async function unregisterHotKey(keys:string) {
    console.log(`unregister ${keys} ing`)
    try {
      if (await isRegistered(keys)) {
        await unregister(keys)
      }
    } catch(e) {
      toast.error(`unregister err: ${e}`)
      return false
    }
    return true
  }
  interface ShortcutClient {
    clientName: string,
    use: string,
    func: ShortcutHandler
  }

  const shortcutClient = ref<ShortcutClient[]>([])
  function generateShortcutCallback(use: string) {

    return (event: ShortcutEvent) => {
      if (event.state == "Pressed")
        return
      console.log(event)
      let tmp = null
      for (const v of settings.hotKeyList) {
        if (v.use == use) {
          tmp = v
          break
        }
      }
      if (tmp == null) {
        throw new Error(`can not find use ${use}`)
      }

      for (const client of shortcutClient.value) {
        if (client.use == tmp.use) {
          client.func(event)
        }
      }
    }

  }

  async function enableShortcut(use:string, enable: boolean, keys: null|string) {
    let tmp = null
    for (const v of settings.hotKeyList) {
      if (v.use == use) {
        tmp = v
        break
      }
    }
    if (tmp == null) {
      throw new Error(`can not find use ${use}`)
    }

    if (!enable) {
      if (await unregisterHotKey(tmp.keys))
        tmp.isOpen = false
      if (keys) {
        tmp.keys = keys
      }
    } else {
      if (keys) {
        if (await unregisterHotKey(tmp.keys) && await registerHotKey(keys, generateShortcutCallback(use))) {
          tmp.isOpen = true
          tmp.keys = keys
        }
      } else {
        if (await registerHotKey(tmp.keys, generateShortcutCallback(use)))
          tmp.isOpen = true
      }
    }
  }

  async function modifyHotKey(use:string, keys: string) {
    let tmp = null
    for (const v of settings.hotKeyList) {
      if (v.use == use) {
        tmp = v
        break
      }
    }
    if (tmp == null) {
      throw new Error(`can not find use ${use}`)
    }
    if (tmp.isOpen) {
      if (await unregisterHotKey(tmp.keys) && await registerHotKey(keys, generateShortcutCallback(use))) {
        tmp.keys = keys
        return true
      }
    } else {
      tmp.keys = keys
      return true
    }
    return false
  }

  function registerHotKeyClient(client: string, use: Uses, func: ShortcutHandler) {
    let flag = false
    for (const v of shortcutClient.value) {
      if (v.clientName == client && v.use == use) {
        v.func = func
        flag = true
      }
    }
    if (!flag) {
      shortcutClient.value.push({
        clientName: client,
        use,
        func
      })
    }
  }

  const appWindows = new Window('main')
  const pinState = ref(false)
  async function setAlwaysTop(v: boolean) {
    try {
      appWindows.setAlwaysOnTop(v)
      pinState.value = v
    } catch(e) {
      toast.error(`pin err: ${e}`)
    }
  }

  async function winToTop() {
    try {
      if (!pinState.value) {
        await appWindows.setAlwaysOnTop(true)
        await appWindows.setAlwaysOnTop(false)
      }
    } catch(e) {
      toast.error(`pin err: ${e}`)
    }
  }

  async function showWin() {
    try {
      await appWindows.show()
      if (!pinState.value) {
        await appWindows.setAlwaysOnTop(true)
        await appWindows.setAlwaysOnTop(false)
      }
    } catch(e) {
      toast.error(`pin err: ${e}`)
    }
  }
  return {
    settings,apiLangs,apiList,shortcutClient,pinState, getStoreSetting, mapKey2Str,chooseApi, registerHotKeyClient, enableShortcut, modifyHotKey, allBackendInit, setAlwaysTop, winToTop, showWin
  }
})