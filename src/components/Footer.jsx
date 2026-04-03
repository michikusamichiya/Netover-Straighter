export default function Header() {
  return (
    <footer>
      <div className="w-full py-8 px-6 bg-neutral-100 text-neutral-600 text-sm flex flex-col items-center gap-2 border-t border-neutral-200">
        <div>
          &copy; {new Date().getFullYear()} FilterKiller. All rights reserved.
        </div>
        <div className="flex gap-4">
          <a
            href="https://github.com/FilterKiller"
            target="_blank"
            rel="noopener noreferrer"
            className="hover:underline"
          >
            GitHub
          </a>
          <a
            href="/about"
            className="hover:underline"
          >
            About
          </a>
          <a
            href="/members"
            className="hover:underline"
          >
            Members
          </a>
        </div>
      </div>
    </footer>
  );
}
