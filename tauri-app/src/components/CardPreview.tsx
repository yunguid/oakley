import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';

interface CardJson {
  id: number;
  front: string;
  back: string;
  tags: string[];
}

interface Props {
  card: CardJson;
  onClose: () => void;
}

const CardPreview: React.FC<Props> = ({ card, onClose }) => {
  const [front, setFront] = useState(card.front);
  const [back, setBack] = useState(card.back);

  const save = async () => {
    await invoke('accept_card', { card: { ...card, front, back } });
    onClose();
  };
  const discard = async () => {
    await invoke('discard_card', { cardId: card.id });
    onClose();
  };

  return (
    <div className="flex flex-col gap-10 text-neutral-800">
      {/* Front */}
      <div className="flex flex-col gap-4">
        <label htmlFor="front" className="text-[11px] tracking-[0.2em] uppercase text-neutral-400">
          Front
        </label>
        <textarea
          id="front"
          className="w-full h-28 bg-transparent border-b border-neutral-200 hover:border-neutral-300 focus:border-neutral-900 transition-colors duration-200 focus:ring-0 text-lg leading-relaxed placeholder-neutral-300"
          value={front}
          onChange={(e) => setFront(e.target.value)}
        />
      </div>

      {/* Back */}
      <div className="flex flex-col gap-4">
        <label htmlFor="back" className="text-[11px] tracking-[0.2em] uppercase text-neutral-400">
          Back
        </label>
        <textarea
          id="back"
          className="w-full h-28 bg-transparent border-b border-neutral-200 hover:border-neutral-300 focus:border-neutral-900 transition-colors duration-200 focus:ring-0 text-lg leading-relaxed placeholder-neutral-300"
          value={back}
          onChange={(e) => setBack(e.target.value)}
        />
      </div>

      {/* Actions */}
      <div className="flex gap-4 justify-end pt-2">
        <button
          className="px-5 py-2.5 text-sm font-medium text-neutral-400 hover:text-neutral-900 transition-colors duration-200"
          onClick={discard}
        >
          Discard
        </button>
        <button
          className="px-6 py-2.5 text-sm font-medium rounded-full bg-neutral-900 hover:bg-neutral-800 active:scale-[.98] transition-all duration-200 text-white/90 hover:text-white shadow-[0_2px_8px_rgba(0,0,0,0.08)] hover:shadow-[0_2px_12px_rgba(0,0,0,0.12)]"
          onClick={save}
        >
          Save
        </button>
      </div>
    </div>
  );
};

export default CardPreview; 