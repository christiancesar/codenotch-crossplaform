import { AnimatePresence, motion } from "motion/react";
import { notchMotion } from "@/libs/motion";

/** One line at the bottom of the notch window: a refused second instance, an unreadable config. */
export function NoticeToast({ message }: { message: string | null }) {
  return (
    <AnimatePresence>
      {message && (
        <motion.div
          role="status"
          className="rounded-lg bg-notch/90 ring-1 ring-notch-outline px-2 py-1 text-[10px] text-muted-foreground"
          initial={{ opacity: 0, y: 6 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: 6 }}
          transition={notchMotion.contents}
        >
          {message}
        </motion.div>
      )}
    </AnimatePresence>
  );
}
