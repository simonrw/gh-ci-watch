import { StorageContext } from "@/lib/storage";
import { MouseEventHandler, useContext } from "react";
import { Button } from "./ui/button";
import { LogOut } from "lucide-react";
import { useNavigate } from "react-router-dom";

export function LogoutButton() {
  const storage = useContext(StorageContext);
  const naviagte = useNavigate();

  const logout: MouseEventHandler<HTMLButtonElement> = (e) => {
    e.preventDefault();
    storage.reset();
    naviagte("/auth");
  };

  return (
    <Button variant="outline" size="icon" onClick={logout}>
      <LogOut />
    </Button>
  );
}
