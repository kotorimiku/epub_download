import { Result, CommandError } from '../bindings';

export function useRunCommand() {
  async function runCommand<T>(params: {
    command: () => Promise<Result<T, CommandError>>;
    onSuccess?: (data: T) => void | Promise<void>;
    errMsg?: string;
    onError?: (err: CommandError) => void | Promise<void>;
    onFinally?: () => void | Promise<void>;
  }): Promise<T | null> {
    const {
      command,
      onSuccess,
      errMsg = '请求失败',
      onError,
      onFinally,
    } = params;

    try {
      const res = await command();
      if (res.status === 'ok') {
        await onSuccess?.(res.data);
        return res.data;
      } else {
        throw new Error(res.error);
      }
    } catch (err) {
      console.error(`${errMsg}：`, err);
      onError?.(err as CommandError);
    } finally {
      onFinally?.();
    }
    return null;
  }

  return runCommand;
}
