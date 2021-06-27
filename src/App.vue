<template>
  <div class="flex flex-col items-stretch">
    <RecordingActions class="mt-8" />
    <RecordingOptions />
    <KapturePlayback />
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue';
import RecordingOptions from './components/RecordingSettings.vue';
import RecordingActions from './components/RecordingActions.vue';
import KapturePlayback from './components/KapturePlayback.vue';

export default defineComponent({
  name: 'App',
  components: {
    RecordingActions,
    RecordingOptions,
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
