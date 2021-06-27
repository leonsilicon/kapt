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
    <button v-if="isKaptActivated" class="bg-blue-400 p-2 rounded-lg" @click="createKapture">
      Create Kapture
    </button>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { readBinaryFile } from '@tauri-apps/api/fs';
import { state } from './state';

export default defineComponent({
  setup() {
    const isKaptActivated = ref(false);

    async function activateKapt() {
      isKaptActivated.value = true;
      await invoke('activate_kapt');
    }

    async function deactivateKapt() {
      await invoke('deactivate_kapt');
      isKaptActivated.value = false;
    }

    async function createKapture() {
      const kapturePath: string = await invoke('create_kapture', {
        timestamp: new Date().getTime(),
      });
      readBinaryFile(kapturePath).then((video) => {
        const intArray = new Uint8Array(video);
        const objectUrl = URL.createObjectURL(
          new Blob([intArray], {
            type: 'video/mp4',
          })
        );
        state.kaptureObjectUrl = objectUrl;
      });
    }

    return {
      createKapture,
      isKaptActivated,
      activateKapt,
      deactivateKapt,
    };
  },
});
</script>
