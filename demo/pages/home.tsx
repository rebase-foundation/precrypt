interface Listing {
  title: string;
  description: string;
  screenshots: Array<{
    url: string;
  }>;
}

const listings: Array<Listing> = [
  {
    title: 'hi there',
    description: 'hello there',
    screenshots: [
      {
        url: 'https://',
      },
    ],
  },
];

export default function Page() {
  let pieces = listings.map((l) => {
    <div>
      <div>{l.title}</div>
    </div>;
  });

  return <div>{pieces}</div>;
}
