import React, { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import CardPreview from './components/CardPreview';

interface CardJson {
  id: number;
  front: string;
  back: string;
  tags: string[];
}

export default function App() {
  const [loading, setLoading] = useState(false);
  const [card, setCard] = useState<CardJson | null>(null);
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    // Skip when running in a regular browser (Tauri APIs unavailable)
    if (!(window as any).__TAURI_IPC__) return;

    const unlisten: Array<() => void> = [];

    const setup = async () => {
      // Show spinner when hotkey fires (legacy path)
      unlisten.push(
        await listen('hotkey', () => {
          setLoading(true);
          setVisible(true);
        })
      );

      // Primary path – backend emits card_created after OCR/LLM pipeline
      unlisten.push(
        await listen<CardJson>('card_created', (event) => {
          setCard(event.payload as CardJson);
          setLoading(false);
          setVisible(true);
        })
      );
    };

    setup();

    return () => {
      unlisten.forEach((u) => u());
    };
  }, []);

  if (!visible) return null;

  return (
    <div className="fixed inset-0 flex items-center justify-center">
      <div className="w-[460px] p-10 bg-white/90 backdrop-blur-xl rounded-3xl shadow-[0_8px_32px_rgba(0,0,0,0.08)] border border-white/20 animate-fade-in">
        {loading ? (
          <div className="flex flex-col items-center justify-center h-40 text-neutral-600">
            <div className="animate-spin rounded-full h-8 w-8 border-2 border-neutral-200 border-t-neutral-800" />
            <p className="mt-4 text-sm tracking-wide">Generating card…</p>
          </div>
        ) : card ? (
          <CardPreview card={card} onClose={() => setVisible(false)} />
        ) : null}
      </div>
    </div>
  );
} 