<script setup lang="ts">
import { Undo2 } from 'lucide-vue-next';
import { useRouter } from 'vue-router';
import Button from '@/components/ui/button/Button.vue';

const router = useRouter()

</script>

<template>
<div class="relative h-full w-full flex flex-col">
  <div class="flex flex-row">
    <Button class="absolute right-2 top-2" variant="ghost" @click="router.back()" ><undo2></undo2></Button>
  </div>
</div>
</template>