import React from "react";

export default function Warning({ title, message }) {
  return (
    <div className="bg-red-500 text-white px-4 py-2 rounded-md my-2 bg-yellow-800 border-l-4 border-yellow-500">
      <h2 className="text-xl font-bold text-yellow-100">{title}</h2>
      <p className="text-sm py-2">{message}</p>
    </div>
  )
}