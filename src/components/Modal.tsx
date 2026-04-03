import type { ReactNode } from "react";

interface ModalProps {
  flag: boolean;
  setFlag?: (flag: boolean) => void;
  dom?: ReactNode;
  children?: ReactNode;
  title?: string;
  showClose?: boolean;
  className?: string;
  contentClassName?: string;
}

export function Modal({
  flag,
  setFlag = () => {},
  dom,
  children,
  title,
  showClose = true,
  className = "",
  contentClassName = "",
}: ModalProps) {
  if (!flag) return null;
  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-netover_text bg-opacity-60"
      onClick={() => setFlag(false)}
      style={{ backdropFilter: "blur(2px)" }}
    >
      <div
        className={`relative bg-netover_text text-netover_bg rounded-lg shadow-lg p-6 min-w-[320px] max-w-full w-fit ${className}`}
        onClick={(e) => e.stopPropagation()}
      >
        {showClose && (
          <button
            type="button"
            aria-label="Close"
            className="absolute top-2 right-3 text-2xl font-bold text-netover_bg hover:text-gray-300"
            onClick={() => setFlag(false)}
          >
            ×
          </button>
        )}
        {title && (
          <div className="text-xl mb-4 font-semibold">{title}</div>
        )}
        <div className={`modal-content ${contentClassName}`}>
          {dom}
          {children}
        </div>
      </div>
    </div>
  );
}
