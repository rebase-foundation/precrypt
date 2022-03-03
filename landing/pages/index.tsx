import type { NextPage } from 'next';

const Home: NextPage = () => {
  return (
    <div>
      {/* <!-- HEADER --> */}
      <div className="border-b">
        <div className='px-4 py-4 mx-auto max-w-2xl flex flex-flex items-center justify-between'>
          <a className='font-bold text-center text-2xl m-0 p-0'>
            Precrypt
          </a>
          <div className='flex flex-row gap-5'>
            <a
              target={"_blank"}
              href="https://discord.gg/FBkAZ4cv"
              className=" font-bold text-sm px-2 flex justify-between items-center p-1 rounded-sm  hover:opacity-50 cursor-pointer"
            >
              <svg
                className="h-10"
                width="24"
                height="14"
                viewBox="0 0 71 55"
                fill="none"
                xmlns="http://www.w3.org/2000/svg"
              >
                <g clip-path="url(#clip0)">
                  <path
                    d="M60.1045 4.8978C55.5792 2.8214 50.7265 1.2916 45.6527 0.41542C45.5603 0.39851 45.468 0.440769 45.4204 0.525289C44.7963 1.6353 44.105 3.0834 43.6209 4.2216C38.1637 3.4046 32.7345 3.4046 27.3892 4.2216C26.905 3.0581 26.1886 1.6353 25.5617 0.525289C25.5141 0.443589 25.4218 0.40133 25.3294 0.41542C20.2584 1.2888 15.4057 2.8186 10.8776 4.8978C10.8384 4.9147 10.8048 4.9429 10.7825 4.9795C1.57795 18.7309 -0.943561 32.1443 0.293408 45.3914C0.299005 45.4562 0.335386 45.5182 0.385761 45.5576C6.45866 50.0174 12.3413 52.7249 18.1147 54.5195C18.2071 54.5477 18.305 54.5139 18.3638 54.4378C19.7295 52.5728 20.9469 50.6063 21.9907 48.5383C22.0523 48.4172 21.9935 48.2735 21.8676 48.2256C19.9366 47.4931 18.0979 46.6 16.3292 45.5858C16.1893 45.5041 16.1781 45.304 16.3068 45.2082C16.679 44.9293 17.0513 44.6391 17.4067 44.3461C17.471 44.2926 17.5606 44.2813 17.6362 44.3151C29.2558 49.6202 41.8354 49.6202 53.3179 44.3151C53.3935 44.2785 53.4831 44.2898 53.5502 44.3433C53.9057 44.6363 54.2779 44.9293 54.6529 45.2082C54.7816 45.304 54.7732 45.5041 54.6333 45.5858C52.8646 46.6197 51.0259 47.4931 49.0921 48.2228C48.9662 48.2707 48.9102 48.4172 48.9718 48.5383C50.038 50.6034 51.2554 52.5699 52.5959 54.435C52.6519 54.5139 52.7526 54.5477 52.845 54.5195C58.6464 52.7249 64.529 50.0174 70.6019 45.5576C70.6551 45.5182 70.6887 45.459 70.6943 45.3942C72.1747 30.0791 68.2147 16.7757 60.1968 4.9823C60.1772 4.9429 60.1437 4.9147 60.1045 4.8978ZM23.7259 37.3253C20.2276 37.3253 17.3451 34.1136 17.3451 30.1693C17.3451 26.225 20.1717 23.0133 23.7259 23.0133C27.308 23.0133 30.1626 26.2532 30.1066 30.1693C30.1066 34.1136 27.28 37.3253 23.7259 37.3253ZM47.3178 37.3253C43.8196 37.3253 40.9371 34.1136 40.9371 30.1693C40.9371 26.225 43.7636 23.0133 47.3178 23.0133C50.9 23.0133 53.7545 26.2532 53.6986 30.1693C53.6986 34.1136 50.9 37.3253 47.3178 37.3253Z"
                    fill="#000000"
                  />
                </g>
                <defs>
                  <clipPath id="clip0">
                    <rect width="71" height="55" fill="white" />
                  </clipPath>
                </defs>
              </svg>
            </a>
            <a
              target={"_blank"}
              href="https://github.com/rebase-foundation/precrypt"
              className="font-bold text-sm px-2 flex justify-between items-center p-1 rounded-sm hover:opacity-50 cursor-pointer"
            >
              <img src="/github.svg" height={14} width={24} className="h-10" />
            </a>
          </div>
        </div>
      </div>

      {/* <!-- Body --> */}
      <div className="px-4 py-10 mx-auto max-w-2xl">
        {/* <!-- Intro --> */}
        <h1 className="font-bold text-xl">Permissioned files for distributed projects</h1>
        <p className="pt-3 text-lg">
          Distributed projects pair best with distributed storage solutions like <a className="text-blue-500 underline" href="https://ipfs.io">IPFS</a>. By nature, these solutions store files on
          public infrastructure where anyone can access them. Precrypt allows files to be encrypted at rest and decrypted only by users with permission to do so. For example, a game developer on
          <a className="text-blue-500 underline" href="https://strangemood.org"> Strangemood</a> can store encrypted files on IPFS that are only decryptable by purchasers of the game.
        </p>
        <div className="pt-3 flex flex-row gap-5 flex-wrap">
          <a target="_blank" href="https://demo.precrypt.org">
            <button className="border-2 rounded p-2 font-bold text-lg">
              Play with the demo
            </button>
          </a>
          <a target="_blank" href="https://docs.rs/precrypt/latest/precrypt/index.html">
            <button className="border-2 rounded p-2 font-bold text-lg">
              Read the docs
            </button>
          </a>
        </div>

        {/* <!-- How it works --> */}
        <h1 className="pt-10 font-bold text-xl">How it works</h1>
        <p className="pt-3 text-lg">
          Precrypt works using <b>proxy based re-encryption</b>. Alice encrypts her file and gives a special re-encryption key to Paul (the trusted proxy), along with rules about who can access the
          file. Bob can give his public key to Paul and ask for access to the file. If Paul determines Bob has permission, he can generate a decryption key that is unique to Bob. Bob can then use the
          decryption key with his private key to decrypt the encrypted file. This approach has the following advantages:
        </p>
        <ul className="text-lg  list-disc list-inside">
          <li className="pt-3">Files are encrypted <b>once</b> and can be decrypted in the future by public keys not known at encryption time.</li>
          <li>Decryption keys are <b>unique</b> to and only usable by the key pair they were created for.</li>
          <li>The trusted proxy <b>never needs access</b> to the decrypted or encrypted file.</li>
        </ul>
      </div>
    </div>
  );
};

export default Home;
