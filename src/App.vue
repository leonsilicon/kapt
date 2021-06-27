<template>
  <div class="flex flex-col items-stretch">
    <RecordingActions class="mt-8 mb-4" />
    <RecordingSettings />
    <KapturePlayback />
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue';
import RecordingSettings from './components/RecordingSettings.vue';
import RecordingActions from './components/RecordingActions.vue';

export default defineComponent({
  name: 'App',
  components: {
    RecordingActions,
    RecordingSettings,
  },
});
</script>

<style>
#app {
  font-family: Avenir, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-align: center;
  color: #2c3e50;
}

* {
  box-sizing: border-box;
}
</style>
