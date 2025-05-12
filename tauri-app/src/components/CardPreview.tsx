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
    <div className="flex flex-col gap-8 text-neutral-800">
      {/* Front */}
      <div className="flex flex-col gap-3">
        <label htmlFor="front" className="text-xs tracking-widest uppercase text-neutral-500">
          Front
        </label>
        <textarea
          id="front"
          className="w-full h-28 bg-transparent border-b border-neutral-300 focus:border-black/80 focus:ring-0 text-lg leading-relaxed placeholder-neutral-400/70"
          value={front}
          onChange={(e) => setFront(e.target.value)}
        />
      </div>

      {/* Back */}
      <div className="flex flex-col gap-3">
        <label htmlFor="back" className="text-xs tracking-widest uppercase text-neutral-500">
          Back
        </label>
        <textarea
          id="back"
          className="w-full h-28 bg-transparent border-b border-neutral-300 focus:border-black/80 focus:ring-0 text-lg leading-relaxed placeholder-neutral-400/70"
          value={back}
          onChange={(e) => setBack(e.target.value)}
        />
      </div>

      {/* Actions */}
      <div className="flex gap-3 justify-end pt-4">
        <button
          className="px-4 py-2 text-sm font-medium text-neutral-500 hover:text-neutral-800 transition-colors"
          onClick={discard}
        >
          Discard
        </button>
        <button
          className="px-5 py-2 text-sm font-semibold rounded-full bg-neutral-900 hover:bg-neutral-800 active:scale-[.98] transition-transform text-white shadow-sm"
          onClick={save}
        >
          Save
        </button>
      </div>
    </div>
  );
};

export default CardPreview; 