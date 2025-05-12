import React from 'react';

export interface CardJson {
  id: number;
  front: string;
  back: string;
  tags: string[];
}

interface Props {
  cards: CardJson[];
}

const CardList: React.FC<Props> = ({ cards }) => {
  return (
    <div className="px-10 py-8 grid gap-6 grid-cols-1 md:grid-cols-2 lg:grid-cols-3">
      {cards.map((c) => (
        <div key={c.id} className="p-6 rounded-2xl bg-white/90 backdrop-blur-sm shadow border border-white/30">
          <h3 className="text-sm tracking-wider text-neutral-400 mb-3">#{c.id}</h3>
          <p className="font-semibold text-neutral-800 mb-2 whitespace-pre-wrap break-words">{c.front}</p>
          <p className="text-neutral-600 whitespace-pre-wrap break-words">{c.back}</p>
          {c.tags.length > 0 && (
            <div className="flex flex-wrap gap-2 mt-4">
              {c.tags.map((t) => (
                <span key={t} className="text-xs bg-neutral-200/60 text-neutral-600 px-2 py-0.5 rounded-full">
                  {t}
                </span>
              ))}
            </div>
          )}
        </div>
      ))}
    </div>
  );
};

export default CardList; 