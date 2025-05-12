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
    <div className="flex flex-col space-y-6 text-gray-800">
      <div className="space-y-2">
        <label className="text-sm font-semibold" htmlFor="front">Front</label>
        <textarea
          id="front"
          className="w-full h-28 border border-gray-300 rounded-lg p-3 resize-none focus:outline-none focus:ring-2 focus:ring-indigo-400 shadow-inner bg-white/80 backdrop-blur-sm"
          value={front}
          onChange={(e) => setFront(e.target.value)}
        />
      </div>

      <div className="space-y-2">
        <label className="text-sm font-semibold" htmlFor="back">Back</label>
        <textarea
          id="back"
          className="w-full h-28 border border-gray-300 rounded-lg p-3 resize-none focus:outline-none focus:ring-2 focus:ring-indigo-400 shadow-inner bg-white/80 backdrop-blur-sm"
          value={back}
          onChange={(e) => setBack(e.target.value)}
        />
      </div>

      <div className="flex gap-3 justify-end pt-2">
        <button
          className="px-4 py-2 rounded-lg border border-gray-300 hover:bg-gray-100 transition-colors"
          onClick={discard}
        >
          Discard
        </button>
        <button
          className="px-4 py-2 rounded-lg bg-indigo-600 hover:bg-indigo-700 text-white shadow-md transition-colors"
          onClick={save}
        >
          Save
        </button>
      </div>
    </div>
  );
};

export default CardPreview; 