<script setup lang="ts">
import Textarea from '@/components/ui/textarea/Textarea.vue';
import Toggle from '@/components/ui/toggle/Toggle.vue';
import Separator from '@/components/ui/separator/Separator.vue';
import { toast } from 'vue-sonner'
import Button from '@/components/ui/button/Button.vue';
import { invoke } from '@tauri-apps/api/core';
import { computed, reactive, ref, useTemplateRef } from 'vue';
import { onMounted } from 'vue'
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  // SelectLabel,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import { useSettingsStore } from '@/stores/settings';
import { ShortcutEvent } from '@tauri-apps/plugin-global-shortcut';
import { readText, writeText } from '@tauri-apps/plugin-clipboard-manager';
import { Command } from '@tauri-apps/plugin-shell';
import { Copy, LoaderPinwheel } from 'lucide-vue-next';

const settingsStore = useSettingsStore()
const finalText = ref("");
const transText = ref("");
const btnDis = ref(false);

const from = ref('')
const to = ref('')

function throttle<T extends (...args: any[]) => void>(
  fn: T,
  delay: number
): (...args: Parameters<T>) => void {
  let last = 0

  return (...args: Parameters<T>) => {
    const now = Date.now()
    console.log(now - last)
    if (now - last >= delay) {
      last = now
      fn(...args)
    } else {
      toast.error("太频繁了")
    }
  }
}

async function copyToClip() {
  if (transText.value.length) {
    await writeText(transText.value);
    toast.info("copy to clip!")
  }
}

async function transitate(text: string) {
  if (!settingsStore.settings.defaultApi) {
    toast.error("choose api!")
    return
  }
  btnDis.value = true;
  try {
    let data = {
      'originText': text,
      'from': from.value?from.value:null,
      'to': to.value?to.value:null
    }
    console.log(data)
    let res = await invoke<string>('translate', data)
    transText.value = res;
  } catch (e) {
    console.error(e)
    toast.error(String(e))
  }
  btnDis.value = false;
}

let translitClick = throttle(()=> {
  if (finalText.value.length != 0) {
    transitate(finalText.value)
  }
}, 1000)

const langFrom = computed(()=>{
  return settingsStore.apiLangs
})

const langTo = computed(()=>{
  return settingsStore.apiLangs.filter((v)=>v[0] != from.value)
})

function textSelect(e: Event) {
  const target = e.target as HTMLTextAreaElement
  const selectedText = target.value.substring(
    target.selectionStart,
    target.selectionEnd
  )
  
  console.log(selectedText)
}

async function callOfSelect(e: ShortcutEvent) {
  console.log(e)
  settingsStore.showWin()
  try {
    let result = await Command.create('exec-xclip', [
      'xclip',
      "-o",
    ]).execute();
    originText.value = result.stdout
    if (settingsStore.settings.autoTrans)
      transitate(_finalText.value)
    toast.info("get text from select")
  } catch(e) {
    toast.error(`failed to exec xclip, only support in x11`)
  }
}

async function callOfClip(e: ShortcutEvent) {
  console.log(e)
  settingsStore.showWin()
  toast.info("copy from clipboard")
  originText.value = await readText();
  if (settingsStore.settings.autoTrans)
    transitate(_finalText.value)
}

const formatObject = reactive([
  {
    toolTip: "去除多余的换行",
    icon: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-wrap-text-icon lucide-wrap-text">
          <path d="m16 16-2 2 2 2"></path>
          <path d="M3 12h15a3 3 0 1 1 0 6h-4"></path>
          <path d="M3 18h7"></path>
          <path d="M3 6h18"></path>
          <path d="m21.21346,2.39328l-18.20221,18.65164"/>
          </svg>`,
    func: (text: string) => {
      const r1 = /\n+/g;
      const r2 = /(?<=\p{L}) *\n *(?=\p{L})/gu
      return text.replace(r1, "\n").replace(r2, "").replace(/[ ]+\n(?=\p{L})/gu, " ")
    },
    isOpen: false
  },
  {
    toolTip: "去除多余的空格",
    icon: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-space-icon lucide-space"><path d="M22 17v1c0 .5-.5 1-1 1H3c-.5 0-1-.5-1-1v-1"/><path d="m21.21346,2.39328l-18.20221,18.65164"/></svg>`,
    func: (text: string) => {
      return text.replace(/ +/g," ")
    },
    isOpen: false
  },
])

let originText = ref("")
const originTextarea = useTemplateRef<HTMLTextAreaElement>("originTextarea");
const _finalText = computed(()=>{
  let tmp = originText.value
  formatObject.forEach((v)=>{
    if (v.isOpen)
      tmp = v.func(tmp)
  })
  finalText.value = tmp
  console.log(`o:${originText.value}`)
  console.log(`f:${finalText.value}`)
  // const textarea:HTMLTextAreaElement = document.getElementById('ori') as HTMLTextAreaElement
  // console.log(`e: ${textarea.value}`)
  return tmp
})

function formatText(text: string|number) {
  text = text.toString()
  originText.value = text
}

onMounted(()=>{
  console.log("mounted IndexView")
  settingsStore.registerHotKeyClient("index", "Select", throttle(callOfSelect, 1000))
  settingsStore.registerHotKeyClient("index", "Clip", throttle(callOfClip, 1000))
  settingsStore.showWin()
})
</script>

<template>
  <div class="flex flex-col gap-2 p-2 basis-full">
    <div class="flex flex-row gap-2">
      <Select v-model:model-value="from" :disabled="!settingsStore.settings.defaultApi">
        <SelectTrigger>
          <SelectValue placeholder="From" />
        </SelectTrigger>
        <SelectContent>
          <SelectGroup>
            <SelectItem v-for="val in langFrom" :value="val[0]">
              {{ val[1] }}
            </SelectItem>
          </SelectGroup>
        </SelectContent>
      </Select>
      <Select v-model:model-value="to" :disabled="!settingsStore.settings.defaultApi">
        <SelectTrigger>
          <SelectValue placeholder="To" />
        </SelectTrigger>
        <SelectContent>
          <SelectGroup>
            <SelectItem v-for="val in langTo" :value="val[0]">
              {{ val[1] }}
            </SelectItem>
          </SelectGroup>
        </SelectContent>
      </Select>
      <Separator orientation="vertical" />
      <TooltipProvider v-for="(v, _) in formatObject">
        <Tooltip>
          <TooltipTrigger as="div" class="flex items-center">
            <Toggle variant="outline" size="sm" v-model:model-value="v.isOpen"  v-html="v.icon"></Toggle>
          </TooltipTrigger>
          <TooltipContent>
            <p v-text="v.toolTip"></p>
          </TooltipContent>
        </Tooltip>
      </TooltipProvider>

        <Button class="ml-auto" @click="translitClick" :disabled="btnDis">
          <LoaderPinwheel class="" :class="{'animate-spin': btnDis}"/>
          Go!
      </Button>

    </div>
    <div class="flex flex-col md:flex-row gap-2 basis-full relative">
      <Textarea id="ori" ref="originTextarea" class="overflow-y-auto resize-y h-50 md:resize-none md:h-full" placeholder="Origin text"
       :model-value="_finalText" @update:model-value="formatText" @select="textSelect"></Textarea>
      <Textarea class="overflow-y-auto resize-none grow" placeholder="Waiting for translation"
        v-model:model-value="transText"></Textarea>
      <TooltipProvider >
        <Tooltip>
          <TooltipTrigger as-child>
            <Button class="absolute bottom-2 right-2 opacity-50 hover:opacity-100" variant="outline" size="icon" @click="copyToClip"><Copy /></Button>
          </TooltipTrigger>
          <TooltipContent>
            <p>copy</p>
          </TooltipContent>
        </Tooltip>
      </TooltipProvider>
    </div>

  </div>
</template>