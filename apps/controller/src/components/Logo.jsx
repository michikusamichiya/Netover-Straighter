export default function Logo({ className = "" }) {
  return (
    <svg
      className={className}
      viewBox="0 0 200 200"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <circle cx="100" cy="100" r="80" stroke="currentColor" strokeWidth="12" />
      {/* 切れ込み部分（例） */}
      <path
        d="M100 20 L140 100 L100 180 L60 100 Z"
        fill="currentColor"
      />
    </svg>
  );
}
