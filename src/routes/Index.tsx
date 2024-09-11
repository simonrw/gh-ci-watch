import { listen } from "@tauri-apps/api/event";
import { useEffect } from "react";

type Event = {
  name: string;
};

export default function Index() {
  useEffect(() => {
    console.log("listening for events");
    const unlisten = listen<Event>("click", ({ payload }) => {
      console.log({ payload });
    });

    return () => {
      unlisten.then((f) => {
        console.log("detaching listener");
        f();
      });
    };
  }, []);

  return (
    <div className="mx-auto bg-slate-950 p-2">
      <h1 className="text-2xl text-center">Actions</h1>
    </div>
  );
}
