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
    <div className="fixed inset-0 flex items-center justify-center pointer-events-none bg-white/40 backdrop-blur-sm">
      <div className="pointer-events-auto bg-white/95 border border-neutral-200 rounded-2xl shadow-xl p-10 w-[460px]">
        {loading ? (
          <div className="flex flex-col items-center justify-center h-40 text-gray-700">
            <div className="animate-spin rounded-full h-10 w-10 border-b-2 border-indigo-600" />
            <p className="mt-4">Generating card…</p>
          </div>
        ) : card ? (
          <CardPreview card={card} onClose={() => setVisible(false)} />
        ) : null}
      </div>
    </div>
  );
} 