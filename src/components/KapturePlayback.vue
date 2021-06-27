<template>
  <video class="max-w-6xl" controls :src="objectUrl"></video>
</template>

<script lang="ts">
import { defineComponent } from 'vue';

export default defineComponent({
  props: {
    objectUrl: {
      type: String,
      required: true,
    },
  },
});
</script>
