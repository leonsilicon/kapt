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
    <div class="text-xl font-bold mt-4">Devices</div>
    <select name="select" v-model="selectedAudioSource">
      <option v-for="source in audioSources" :value="source.id" :key="source.id">
        {{ source.description }}
      </option>
    </select>
    <div class="text-xl font-bold mt-6">Latest Kapture</div>
    <video
      class="max-w-6xl"
      v-if="latestKaptureObjectUrl !== null"
      controls
      :src="latestKaptureObjectUrl"
    ></video>
  </div>
</template>

<script lang="ts">
import { ref, defineComponent, Ref } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import { readBinaryFile } from '@tauri-apps/api/fs';

export default defineComponent({
  name: 'HelloWorld',
  setup() {
    const isKaptActivated = ref(false);
    const latestKaptureObjectUrl = ref<string | null>(null);
    listen('message', (msg) => {
      console.log(msg);
    });

    listen('audio_devices', (msg) => {
      console.log('audio devices', msg);
    });

    type AudioSource = {
      description: string;
      id: number;
    };

    const selectedAudioSource = ref();
    const audioSources: Ref<AudioSource[]> = ref([]);
    invoke('get_audio_sources').then((sources) => {
      audioSources.value = sources as AudioSource[];
      selectedAudioSource.value = audioSources.value[0].id;
    });

    async function activateKapt() {
      isKaptActivated.value = true;
      console.log(selectedAudioSource.value);
      await invoke('activate_kapt', {
        audioSource: selectedAudioSource.value,
      });
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
        console.log(objectUrl);
        latestKaptureObjectUrl.value = objectUrl;
      });
    }

    return {
      isKaptActivated,
      latestKaptureObjectUrl,
      activateKapt,
      deactivateKapt,
      createKapture,
      selectedAudioSource,
      audioSources,
    };
  },
});
</script>

<style scoped>
a {
  color: #42b983;
}

label {
  margin: 0 0.5em;
  font-weight: bold;
}

code {
  background-color: #eee;
  padding: 2px 4px;
  border-radius: 4px;
  color: #304455;
}
</style>
