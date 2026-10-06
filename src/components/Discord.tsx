import { ExternalLink } from "lucide-react";
import { api } from "../lib/ipc";

export const DISCORD_INVITE = "https://discord.gg/unPZxXAtfb";

export function DiscordLogo({ size = 20, className }: { size?: number; className?: string }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="currentColor" className={className} aria-hidden="true">
      <path d="M20.317 4.37a19.79 19.79 0 0 0-4.885-1.515.074.074 0 0 0-.079.037c-.21.375-.444.865-.608 1.25a18.27 18.27 0 0 0-5.487 0 12.64 12.64 0 0 0-.617-1.25.077.077 0 0 0-.079-.037A19.74 19.74 0 0 0 3.677 4.37a.07.07 0 0 0-.032.027C.533 9.046-.32 13.58.099 18.057a.082.082 0 0 0 .031.057 19.9 19.9 0 0 0 5.993 3.03.078.078 0 0 0 .084-.028c.462-.63.874-1.295 1.226-1.994a.076.076 0 0 0-.041-.106 13.1 13.1 0 0 1-1.872-.892.077.077 0 0 1-.008-.128c.126-.094.252-.192.372-.291a.074.074 0 0 1 .078-.01c3.928 1.793 8.18 1.793 12.062 0a.074.074 0 0 1 .078.009c.12.099.246.198.373.292a.077.077 0 0 1-.006.127 12.3 12.3 0 0 1-1.873.892.077.077 0 0 0-.041.107c.36.698.772 1.362 1.225 1.993a.076.076 0 0 0 .084.028 19.84 19.84 0 0 0 6.002-3.03.077.077 0 0 0 .032-.054c.5-5.177-.838-9.674-3.549-13.66a.061.061 0 0 0-.031-.03ZM8.02 15.33c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.956-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.956 2.418-2.157 2.418Zm7.975 0c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.955-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.946 2.418-2.157 2.418Z" />
    </svg>
  );
}

const open = () => api.openUrl(DISCORD_INVITE).catch(() => undefined);

export function DiscordCard() {
  return (
    <button
      type="button"
      onClick={open}
      className="group relative w-full overflow-hidden rounded-2xl text-left text-white shadow-[0_8px_30px_rgba(88,101,242,0.35)] transition-transform duration-150 hover:-translate-y-px active:translate-y-0"
      style={{ background: "linear-gradient(135deg, #5865F2 0%, #4752C4 55%, #3b3f9e 100%)" }}
    >
      <DiscordLogo size={120} className="absolute -right-6 -bottom-8 text-white/10 rotate-[-12deg] transition-transform duration-300 group-hover:rotate-[-4deg] group-hover:scale-105" />
      <div className="absolute inset-0 opacity-60" style={{ background: "radial-gradient(120% 90% at 0% 0%, rgba(255,255,255,0.18), transparent 55%)" }} />
      <div className="relative flex items-center gap-3 px-4 py-3.5">
        <div className="w-11 h-11 rounded-2xl bg-white/15 grid place-items-center shrink-0 shadow-[inset_0_1px_0_rgba(255,255,255,0.25)]">
          <DiscordLogo size={24} />
        </div>
        <div className="min-w-0 flex-1">
          <div className="text-[14px] font-semibold leading-tight">Join our Discord</div>
          <div className="text-[11.5px] text-white/80 leading-snug mt-0.5">Updates, setups, help, and the GPO fishing macro. Come say hi.</div>
        </div>
        <span className="shrink-0 inline-flex items-center gap-1.5 h-8 px-3 rounded-lg bg-white text-[#4752C4] text-[12px] font-semibold shadow-[0_2px_8px_rgba(0,0,0,0.25)] group-hover:bg-white/95">
          Join
          <ExternalLink size={12} />
        </span>
      </div>
    </button>
  );
}
