export function useRunCommand() {
  async function runCommand<T>(params: {
    command: () => Promise<T>;
    onSuccess?: (data: T) => void | Promise<void>;
    errMsg?: string;
    onError?: (err: unknown) => void | Promise<void>;
    onFinally?: () => void | Promise<void>;
  }): Promise<T> {
    const {
      command,
      onSuccess,
      errMsg = 'An error occurred while executing the command.',
      onError,
      onFinally,
    } = params;

    try {
      const res = await command();
      await onSuccess?.(res);
      return res;
    } catch (err) {
      console.error(`${errMsg}:`, err);
      await onError?.(err);
    } finally {
      await onFinally?.();
    }

    return Promise.reject(new Error(errMsg));
  }

  return runCommand;
}
