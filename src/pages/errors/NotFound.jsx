export default function NotFound() {
  return (
    <div className="w-full h-full flex flex-col items-center justify-center text-center p-6">
      <h1 className="text-5xl font-bold">404 - Not Found?</h1>
      <p className="text-neutral-600 mt-2">Something seems possible.</p>

      <a
        href="/"
        className="mt-6 inline-block px-4 py-2 bg-black text-white rounded-lg hover:bg-neutral-800 transition"
      >
        Back to home
      </a>
    </div>
  );
}
