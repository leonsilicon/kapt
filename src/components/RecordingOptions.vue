<template>
  <div class="flex flex-col items-center">
    <div class="flex flex-row mb-2">
      <button class="bg-green-400 p-2 rounded-lg mr-1" @click="startRecording">
        Start recording
      </button>
      <button class="bg-red-400 p-2 rounded-lg ml-1" @click="stopRecording">Stop recording</button>
    </div>
    <button class="bg-blue-400 p-2 rounded-lg" @click="createKapture">Create Kapture</button>
    <div class="text-xl font-bold mt-4">Devices</div>
    <select name="select" v-model="selectedAudioSource">
      <option v-for="source in audioSources" :value="source.id" :key="source.id">
        {{ source.description }}
      </option>
    </select>
  </div>
</template>

<script lang="ts">
import { ref, defineComponent, Ref } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';

export default defineComponent({
  name: 'HelloWorld',
  setup() {
    const isRecording = ref(false);
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

    async function startRecording() {
      isRecording.value = true;
      await invoke('start_recording', {
        audioSource: selectedAudioSource.value,
      });
    }

    async function stopRecording() {
      await invoke('stop_recording');
      isRecording.value = false;
    }

    async function createKapture() {
      const finalPath = await invoke('create_kapture', { timestamp: new Date().getTime() });
      console.log(finalPath);
    }

    return {
      startRecording,
      stopRecording,
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
