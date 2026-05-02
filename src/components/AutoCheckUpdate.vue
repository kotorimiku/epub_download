<template></template>

<script setup lang="ts">
import { autoCheckUpdateNotify } from '@/utils/checkNotify';

onMounted(async () => {
  await autoCheckUpdateNotify();
});
</script>
