<template>
  <div class="flex flex-col items-center my-8">
    <RecordingActions />
    <RecordingSettings class="mt-4" />
    <KapturePlayback />
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue';
import RecordingSettings from './components/RecordingSettings.vue';
import RecordingActions from './components/RecordingActions.vue';
import KapturePlayback from './components/KapturePlayback.vue';

export default defineComponent({
  name: 'App',
  components: {
    RecordingActions,
    RecordingSettings,
    KapturePlayback,
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
