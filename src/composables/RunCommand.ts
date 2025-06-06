import { Result, CommandError } from "../bindings";
import { useNotify } from "./useNotification";

export function useRunCommand() {
  const notify = useNotify();
  async function runCommand<T>(params: {
    command: () => Promise<Result<T, CommandError>>,
    onSuccess?: (data: T) => void,
    errMsg?: string,
    onError?: (err: CommandError) => void
  }) {
    const {
      command,
      onSuccess,
      errMsg = "请求失败",
      onError
    } = params;

    try {
      const res = await command();
      if (res.status === "ok") {
        onSuccess?.(res.data);
      } else {
        throw new Error(res.error);
      }
    } catch (err) {
      console.error(`${errMsg}：`, err);
      if (onError) {
        onError(err as CommandError);
      }else {
        notify.error({ content: `${errMsg}：` + err });
      }
    }
  }

  return runCommand;
}