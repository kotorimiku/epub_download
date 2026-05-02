import { useRunCommand } from '@/composables/useRunCommand';

import { commands } from '../bindings';
import { useNotify } from '../composables/useNotification';

let hasChecked = false;

export async function autoCheckUpdateNotify() {
  if (hasChecked) return;
  hasChecked = true;
  const runCommand = useRunCommand();
  const notify = useNotify();
  let config = await runCommand({ command: commands.getConfigVue });
  if (config.autoCheckUpdate !== false) {
    checkUpdateNotify(notify, runCommand);
  }
}

export function checkUpdateNotify(notify: ReturnType<typeof useNotify> = useNotify(), runCommand: ReturnType<typeof useRunCommand> = useRunCommand()) {
  runCommand({
    command: commands.checkUpdate,
    onSuccess: async (msg) => {
      if (msg.includes('已是最新版本')) {
        return;
      }
      const urlMatch = msg.match(/https?:\/\/[^\s]+/);
      if (urlMatch) {
        const url = urlMatch[0];
        notify.success({
          content: () =>
            h('div', [
              h('div', msg.replace(url, '')),
              h(
                'a',
                {
                  href: url,
                  target: '_blank',
                  style: 'color: #18a058; text-decoration: underline; cursor: pointer;',
                },
                url,
              ),
            ]),
          duration: 0,
        });
      } else {
        notify.success({ content: msg });
      }
    },
    onError: (err) => {
      notify.error({ content: `检查更新失败：${err}` });
    },
  });
}
