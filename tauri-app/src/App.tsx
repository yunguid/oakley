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
    const unlistenAll: Array<() => void> = [];

    const setup = async () => {
      unlistenAll.push(
        await listen('card_generating', () => {
          setLoading(true);
          setVisible(true);
        })
      );
      unlistenAll.push(
        await listen<CardJson>('card_created', (event) => {
          setCard(event.payload);
          setLoading(false);
          setVisible(true);
        })
      );
    };
    setup();

    return () => {
      unlistenAll.forEach((un) => un());
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