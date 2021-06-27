<template>
  <div class="flex flex-col items-center">
    <div class="flex flex-row mb-2">
      <button
        v-if="!isKaptActivated"
        class="bg-green-400 p-2 rounded-lg mr-1"
        @click="activateKapt"
      >
        Activate Kapt
      </button>
      <button v-else class="bg-red-400 p-2 rounded-lg ml-1" @click="deactivateKapt">
        Deactivate Kapt
      </button>
    </div>
    <div
      v-if="isKaptActivated"
      class="border-2 rounded-lg px-4 w-72 h-32 flex flex-col items-center justify-center"
    >
      <div v-if="!isCreateKaptureLoading">
        <div class="text-xl font-bold mt-8">Create Kapture</div>
        <div class="flex flex-row">
          <button
            class="bg-blue-400 rounded-full w-8 h-8 mx-2"
            v-for="seconds in secondsOptions"
            :key="seconds"
            @mouseover="activeSeconds = seconds"
            @mouseleave="activeSeconds = null"
            @click="createKapture(seconds)"
          >
            {{ seconds }}
          </button>
        </div>
        <div
          class="text-sm my-2"
          :style="{ visibility: activeSeconds === null ? 'hidden' : 'visible' }"
        >
          Capture the last {{ activeSeconds }} seconds
        </div>
      </div>
      <div v-else class="flex flex-col items-center">
        <LoadingSpinner />
        <div class="mt-2">Processing Kapture...</div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { readBinaryFile } from '@tauri-apps/api/fs';
import { listen } from '@tauri-apps/api/event';
import { state } from './state';
import LoadingSpinner from './LoadingSpinner.vue';

const secondsOptions = [5, 10, 15, 30, 60];

export default defineComponent({
  components: { LoadingSpinner },
  setup() {
    const isKaptActivated = ref(false);
    listen('kapt_activation_toggled', (data) => {
      isKaptActivated.value = data.payload as boolean;
    });

    const activeSeconds = ref(null);

    async function activateKapt() {
      isKaptActivated.value = true;
      await invoke('activate_kapt');
    }

    async function deactivateKapt() {
      isKaptActivated.value = false;
      await invoke('deactivate_kapt');
    }

    async function onKaptureCreated(kapturePath: string) {
      const videoBytes = await readBinaryFile(kapturePath);
      const intArray = new Uint8Array(videoBytes);
      const objectUrl = URL.createObjectURL(
        new Blob([intArray], {
          type: 'video/mp4',
        })
      );
      state.kaptureObjectUrl = objectUrl;
    }

    listen('kapture_created', async (data) => {
      console.log(data);
      const kapturePath = data.payload as string;
      await onKaptureCreated(kapturePath);
    });

    const isCreateKaptureLoading = ref(false);
    async function createKapture(seconds: number) {
      try {
        isCreateKaptureLoading.value = true;
        const kapturePath: string = await invoke('create_kapture', {
          timestamp: new Date().getTime(),
          secondsToCapture: seconds,
        });
        await onKaptureCreated(kapturePath);
      } finally {
        isCreateKaptureLoading.value = false;
      }
    }

    return {
      createKapture,
      isKaptActivated,
      activateKapt,
      deactivateKapt,
      secondsOptions,
      activeSeconds,
      isCreateKaptureLoading,
    };
  },
});
</script>
