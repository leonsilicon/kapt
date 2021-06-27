import { reactive } from 'vue';

type KaptFrontendState = {
  kaptureObjectUrl: string | null;
};

export const state = reactive<KaptFrontendState>({
  kaptureObjectUrl: null,
});
