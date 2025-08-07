import { listen, emit } from '@tauri-apps/api/event';

export function restoreHtml(callback: (html: string) => void) {
  listen('html', (event) => {
    callback(event.payload as string);
  });
}