<template>
  <button @click="startRecording">Start recording</button>
  <button @click="stopRecording">Stop recording</button>
</template>

<script lang="ts">
import { ref, defineComponent } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';

export default defineComponent({
  name: 'HelloWorld',
  setup() {
    const isRecording = ref(false);
    listen('message', (msg) => {
      console.log(msg);
    });

    async function startRecording() {
      isRecording.value = true;
      await invoke('start_recording');
    }

    async function stopRecording() {
      await invoke('stop_recording');
      isRecording.value = false;
    }

    return {
      startRecording,
      stopRecording
    };
  }
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
