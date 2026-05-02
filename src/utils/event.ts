import { listen } from '@tauri-apps/api/event';

type HtmlEventPayload = {
  requestId: string;
  html: string;
};

export function restoreHtml(callback: (payload: HtmlEventPayload) => void) {
  return listen('html', (event) => {
    callback(event.payload as HtmlEventPayload);
  });
}
