type Props = {
  message: string;
  onRetry?: () => void;
  retryLabel?: string;
};

export function InlineError({ message, onRetry, retryLabel }: Props) {
  return (
    <div className="inline-error" role="alert">
      <span>{message}</span>
      {onRetry && retryLabel ? (
        <button className="inline-error__action" type="button" onClick={onRetry}>
          {retryLabel}
        </button>
      ) : null}
    </div>
  );
}
