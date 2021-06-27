<template>
  <div class="flex flex-col items-center">
    <div class="text-2xl font-bold">Settings</div>
    <div class="max-w-5xl w-full flex flex-col items-center px-8">
      <div>
        Capture the last
        <input
          v-model="secondsToCapture"
          type="number"
          class="pl-2 w-14 border"
          @change="setSecondsToCapture"
        />
        seconds
      </div>

      <div class="flex flex-row mt-2 items-center">
        <div class="text-xl font-bold mr-2 -mb-1">Audio Device</div>
        <select @change="setAudioSource" name="select" v-model="selectedAudioSource">
          <option v-for="source in audioSources" :value="source.id" :key="source.id">
            {{ source.description }}
          </option>
        </select>
      </div>

      <div class="flex flex-row h-7 items-stretch mt-2 w-full">
        <input class="border rounded-md mr-2 px-2 flex-grow" readonly :value="videoFolder" />

        <button class="bg-yellow-400 rounded-md px-4" @click="selectVideoFolder">
          Select Video Folder
        </button>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { ref, defineComponent, Ref } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { homeDir, videoDir } from '@tauri-apps/api/path';

export default defineComponent({
  name: 'HelloWorld',
  setup() {
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

    async function setAudioSource() {
      await invoke('set_audio_source', {
        audioSource: selectedAudioSource.value,
      });
    }

    const videoFolder = ref<string>('');

    async function syncVideoFolder() {
      await invoke('set_video_folder', {
        videoFolder: videoFolder.value,
      });
    }

    (async () => {
      const videoDirectory = await videoDir();

      if (!videoDirectory) {
        const homeDirectory = await homeDir();
        videoFolder.value = homeDirectory;
      } else {
        videoFolder.value = videoDirectory;
      }

      await syncVideoFolder();
    })();

    async function selectVideoFolder() {
      const videoDirectory = await invoke('select_video_folder');
      if (videoDirectory) {
        videoFolder.value = videoDirectory as string;
        await syncVideoFolder();
      }
    }

    const secondsToCapture = ref(15);
    async function setSecondsToCapture() {
      await invoke('set_seconds_to_capture', {
        seconds: secondsToCapture.value,
      });
    }

    return {
      videoFolder,
      selectVideoFolder,
      audioSources,
      selectedAudioSource,
      setAudioSource,
      secondsToCapture,
      setSecondsToCapture,
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
