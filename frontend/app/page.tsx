import { Topbar } from "@/components/Topbar";
import { AnimatedGridPattern } from "@/components/ui/animated-grid-pattern";
import { cn } from "@/lib/utils";

export default function Home() {
  return (
    <div className="fixed w-full h-full">
      <Topbar items={[]}></Topbar>
      <AnimatedGridPattern
        numSquares={50}
        maxOpacity={0.1}
        duration={5}
        repeatDelay={1}
        className={cn(
          "[mask-image:radial-gradient(900px_circle_at_center,white,transparent)]",
          "-inset-x-[6%] -inset-y-[20%] h-[140%] skew-y-12"
        )}
      ></AnimatedGridPattern>
    </div>
  );
}
