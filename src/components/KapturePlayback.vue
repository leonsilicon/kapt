<template>
  <div v-if="objectUrl !== null">
    <div class="text-xl font-bold mt-6">Latest Kapture</div>

    <video class="max-w-6xl" controls :src="state.kaptureObjectUrl"></video>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue';
import { state } from './state';

export default defineComponent({
  setup() {
    return {
      state,
    };
  },
});
</script>
