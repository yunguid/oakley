import React, { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/tauri';
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
      unlisten.push(
        await listen('hotkey', async () => {
          try {
            setLoading(true);
            setVisible(true);

            const text = await navigator.clipboard.readText(); // assumes user copied text prior to Cmd+Shift+P
            const newCard = await invoke<CardJson>('generate_card', { text });
            setCard(newCard as CardJson);
          } catch (err) {
            console.error(err);
          } finally {
            setLoading(false);
          }
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
    <div className="fixed inset-0 flex items-center justify-center pointer-events-none">
      <div className="pointer-events-auto bg-black/70 backdrop-blur rounded-lg shadow-lg p-6 w-[400px]">
        {loading ? (
          <div className="flex flex-col items-center justify-center h-40 text-white">
            <div className="animate-spin rounded-full h-10 w-10 border-b-2 border-white" />
            <p className="mt-4">Generating card…</p>
          </div>
        ) : card ? (
          <CardPreview card={card} onClose={() => setVisible(false)} />
        ) : null}
      </div>
    </div>
  );
} 