import { motion } from "framer-motion";
import { Link } from "react-router-dom";

import logo from "@/assets/Logo.svg";

export default function Home() {
  return (
    <div className="w-full bg-netover_bg text-netover_text">
      {/* HERO */}
      <section className="w-full">
        <div
          className="w-full h-[22rem] flex items-center bg-black"
        >
          <div className="container mx-auto px-6">
            <div className="flex items-center gap-8">
              {/* Rotating logo */}
              <motion.div
                className="w-40 h-40 rounded shadow-2xl bg-black/60 flex items-center justify-center p-3"
                animate={{ rotate: [0, 360] }}
                transition={{ repeat: Infinity, duration: 60, ease: "linear" }}
                whileHover={{ scale: 1.06, transition: { duration: 0.25 } }}
              >
                <img
                  src={logo}
                  alt="NetOver logo"
                  className="w-full h-full object-contain"
                />
              </motion.div>

              <div className="flex-1 text-netover_blue">
                <h1 className="text-5xl sm:text-6xl md:text-7xl font-extrabold leading-tight drop-shadow-lg">
                  NetOver
                </h1>
                <p className="mt-3 text-sm sm:text-base max-w-2xl opacity-95">
                  <strong>"It doesn't matter environments to control released computer"</strong>
                </p>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* MAIN CONTENT */}
      <main className="container mx-auto px-6 -mt-12">
        {/* Purpose card */}
        <section className="bg-netover_bg rounded-xl shadow-2xl p-8 text-center">
          <h2 className="text-3xl md:text-4xl font-semibold mt-4 mb-6 text-netover_text">
            What is the purpose of me?
          </h2>
          <p className="max-w-3xl mx-auto text-sm md:text-base leading-relaxed text-netover_text">
            {/* I believe that introducing filtering software to children is largely
            pointless. Restricting the means by which they can obtain information
            in this way is the same as information control. Children will
            inevitably find a way around it. Instead, the most important thing is
            to think together about how to handle information.
            <br />
            <br />
            As it stands, filtering software still restricts access to useful
            information for no good reason. That's why we decided to provide ways
            for children to obtain information on their own, plus tools, tips, and
            a place to interact with people who share similar goals. */}
            Introducing a remote control tool requires a lot of effort.

            So I created a remote control tool that does not require account creation for basic operation, does not require the installation of any apps on the remote computer, and only performs the minimum amount of external communication necessary.

            These tools could be effective ways to control computers that are free from restrictions.
          </p>
        </section>

        <section className="mt-16">
          <h3 className="text-2xl md:text-3xl text-center font-medium mb-8">
            Special thanks for finding Vulnerability
          </h3>
          No one yet!
        </section>

        {/* Moving */}
        <section className="mt-16">
          <h3 className="text-2xl md:text-3xl text-center font-medium mb-8">
            Precautions when using this site
          </h3>
        <p className="max-w-3xl mx-auto text-sm md:text-base leading-relaxed">
          This site is managing to get by with the free plan, and we are doing our best to make the service available free of charge.<br />
          However, we are not a corporation or NPO, but rather an individual operation. This means that there is a possibility that you may not receive appropriate support, or that the service may be terminated unexpectedly or go down.<br />
          Therefore, we ask for your appropriate use and support. The administrator is not yet of the age of majority in Japan.<br />
          <br />
          If you have any questions, please send an email to "subkusamichiya@gmail.com".
        </p>
        </section>
      </main>
    </div>
  );
}

/* ------------ SERVICE CARD COMPONENT ------------ */
function ServiceCard({
  title,
  subtitle,
  imgSrc,
  children,
  variant = "cyan",
  darkText = false,
  textColorClass = "",
  url
}) {
  const baseText = darkText ? "text-netover_text" : "text-netover_bg";

  const variants = {
    cyan: "bg-gradient-to-br from-emerald-600 to-sky-700",
    amber: "bg-gradient-to-br from-yellow-600 to-emerald-600",
    white: "bg-netover_bg",
    netover: "bg-[#0E0E0E]",
    blue: "bg-blue-500",
  };

  return (
    <motion.article
      className={`rounded-xl p-5 shadow-2xl flex flex-col justify-between ${variants[variant]
        } ${baseText} ${textColorClass}`}
      whileHover={{ scale: 1.03, y: -6 }}
      transition={{ type: "spring", stiffness: 220, damping: 18 }}
    >
      <Link to={url}>
        <div className="text-center">
          <h4 className={`text-xl font-semibold ${baseText}`}>{title}</h4>
          <p className="mt-1 text-sm whitespace-pre-line opacity-90">
            {subtitle}
          </p>
          <img
            src={imgSrc}
            alt={`${title} symbol`}
            className="w-32 h-32 object-contain mx-auto mt-4"
          />
        </div>
        <p className="text-xs mt-4 opacity-80">{children}</p>
      </Link>
    </motion.article>
  );
}

/* ------------ MEMBER CARD ------------ */
function MemberCard({ name, desc, imgSrc }) {
  return (
    <motion.div
      whileHover={{ scale: 1.01 }}
      className="w-96 max-w-full mx-auto bg-white shadow-xl rounded-xl p-6 flex flex-col items-center text-center"
      // w-96: 幅24rem。max-w-fullで親の制約も受ける。
    >
      <img
        src={imgSrc}
        className="w-28 h-28 object-cover rounded-full mb-4"
        alt={name}
      />
      <h4 className="text-xl font-semibold">{name}</h4>
      <p className="text-sm text-neutral-600 mt-2">{desc}</p>
    </motion.div>
  );
}
