// CustomButton.tsx
interface CustomButtonProps {
  text: string;
  onClick?: () => void;
  additionClass?: string;
}

export default function CustomButton({ text, onClick, additionClass = "" }: CustomButtonProps) {
  return (
    <button className={`px-4 py-2 rounded-md my-2 mr-2 hover:bg-netover_blue/80 transition-colors duration-200 ${additionClass}`} onClick={onClick}>
      {text}
    </button>
  )
}
