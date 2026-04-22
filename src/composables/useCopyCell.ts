import { ref } from 'vue';

export function useCopyCell() {
  const copiedKey = ref('');
  const toastVisible = ref(false);
  let toastTimer: ReturnType<typeof setTimeout> | null = null;

  async function copyCell(key: string, value: string) {
    if (!value || value === '—') return;
    try {
      await navigator.clipboard.writeText(value);
    } catch {
      const el = document.createElement('textarea');
      el.value = value;
      document.body.appendChild(el);
      el.select();
      document.execCommand('copy');
      document.body.removeChild(el);
    }
    copiedKey.value = key;
    toastVisible.value = true;
    if (toastTimer) clearTimeout(toastTimer);
    toastTimer = setTimeout(() => {
      copiedKey.value = '';
      toastVisible.value = false;
    }, 1500);
  }

  return { copiedKey, toastVisible, copyCell };
}
