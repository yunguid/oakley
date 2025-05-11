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
    <div className="flex flex-col space-y-4">
      <textarea
        className="w-full h-24 bg-transparent border border-gray-500 rounded text-white font-mono p-2 resize-none"
        value={front}
        onChange={(e) => setFront(e.target.value)}
      />
      <textarea
        className="w-full h-24 bg-transparent border border-gray-500 rounded text-white font-mono p-2 resize-none"
        value={back}
        onChange={(e) => setBack(e.target.value)}
      />
      <div className="flex gap-4 justify-end">
        <button
          className="px-4 py-2 bg-green-600 hover:bg-green-700 text-white rounded"
          onClick={save}
        >
          Save
        </button>
        <button
          className="px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded"
          onClick={discard}
        >
          Discard
        </button>
      </div>
    </div>
  );
};

export default CardPreview; 