import { useNavigate } from "react-router-dom";
import Warning from "../components/Warning.tsx";
import CustomButton from "../components/CustomButton.tsx";

export default function Home() {
  const navigate = useNavigate();
  return (
    <div>
      <h1 className="text-3xl font-bold py-3">NetOver <span className="text-netover_blue">Straighter</span></h1>
      <p className="text-sm py-1">NetOver Straighter is a tool that helps you to control the internet.</p>
      <Warning
        title="Warning"
        message="This is a highly flexible remote control tool that does not require login or special setup. It is provided for free, and since our development team cannot take any responsibility, please use this tool at your own risk."
      />

      <CustomButton text="Launch" onClick={() => {
        navigate("/launch");        
      }} additionClass="bg-netover_blue text-white" />
      <CustomButton text="Pairing" onClick={() => {
        navigate("/pairing");
      }} additionClass="bg-netover_green text-white" />
      <CustomButton text="Manage keys" onClick={() => {
        navigate("/managekey");
      }} additionClass="bg-yellow-600 text-white" />
      <CustomButton text="Settings" onClick={() => {
        navigate("/config");
      }} additionClass="bg-gray-600 text-white" />
    </div>
  )
}